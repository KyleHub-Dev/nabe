package main

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/KyleHub-Dev/nabe/apps/speiche/internal/config"
	"github.com/KyleHub-Dev/nabe/apps/speiche/internal/drivers/adguard"
	"github.com/KyleHub-Dev/nabe/apps/speiche/internal/heartbeat"
)

const version = "0.0.0"

func main() {
	cfg := config.FromEnv()
	client := heartbeat.NewClient(cfg.NabeAPIURL, cfg.EnrollmentToken)
	driver := adguard.NewDriver(cfg.AdGuardURL, cfg.AdGuardUsername, cfg.AdGuardPassword)
	interval, err := strconv.Atoi(cfg.IntervalSeconds)
	if err != nil || interval < 5 {
		interval = 30
	}
	maxInterval, err := strconv.Atoi(cfg.MaxIntervalSeconds)
	if err != nil || maxInterval < interval {
		maxInterval = 300
	}
	backoffAfter, err := strconv.Atoi(cfg.FailureBackoffAfter)
	if err != nil || backoffAfter < 1 {
		backoffAfter = 3
	}

	log.Printf("speiche starting: node=%s api=%s public_server=false", cfg.NodeName, cfg.NabeAPIURL)
	if len(os.Args) > 1 && os.Args[1] == "version" {
		fmt.Println("speiche " + version)
		return
	}

	identity, err := loadOrCreateIdentity(cfg.StateDir, cfg.NodeName)
	if err != nil {
		log.Fatalf("identity setup failed: %v", err)
	}
	if identity.Enrolled && identity.Credential == "" {
		identity.Enrolled = false
		if err := saveIdentity(cfg.StateDir, identity); err != nil {
			log.Fatalf("legacy identity migration failed: %v", err)
		}
	}

	currentInterval := interval
	consecutiveFailures := 0
	for {
		ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
		report := buildReport(ctx, cfg, identity.NodeID, driver)
		var response heartbeat.NodeResponse
		if !identity.Enrolled {
			response, err = client.Enroll(ctx, report)
			if err == nil {
				if response.Credential == "" {
					err = errors.New("central enrollment response omitted edge credential")
				} else {
					identity.Credential = response.Credential
					identity.Enrolled = true
					err = saveIdentity(cfg.StateDir, identity)
				}
			}
		} else {
			response, err = client.Heartbeat(ctx, report, identity.Credential)
		}
		cancel()
		if err != nil {
			if identity.Enrolled && heartbeat.IsForbidden(err) {
				identity.Enrolled = false
				identity.Credential = ""
				if saveErr := saveIdentity(cfg.StateDir, identity); saveErr != nil {
					log.Printf("credential rejection state save failed: %v", saveErr)
				} else {
					log.Printf("edge credential rejected; enrollment will be retried with the bootstrap token")
				}
			}
			consecutiveFailures++
			currentInterval = nextHeartbeatInterval(currentInterval, maxInterval, backoffAfter, consecutiveFailures)
			log.Printf("heartbeat failed: %v; failures=%d next_sleep=%s", err, consecutiveFailures, time.Duration(currentInterval)*time.Second)
		} else {
			if consecutiveFailures > 0 || currentInterval != interval {
				log.Printf("heartbeat recovered after %d failures; reset_sleep=%s", consecutiveFailures, time.Duration(interval)*time.Second)
			}
			consecutiveFailures = 0
			currentInterval = interval
			log.Printf("heartbeat ok: node=%s status=%s last_seen=%s", response.NodeID, response.HealthStatus, response.LastSeenAt)
			telemetryCtx, telemetryCancel := context.WithTimeout(context.Background(), 20*time.Second)
			if telemetryErr := collectAndSendStats(telemetryCtx, client, driver, cfg.StateDir, &identity); telemetryErr != nil {
				log.Printf("telemetry upload failed: %v", telemetryErr)
			}
			telemetryCancel()
		}
		time.Sleep(time.Duration(currentInterval) * time.Second)
	}
}

func nextHeartbeatInterval(current int, max int, backoffAfter int, consecutiveFailures int) int {
	if consecutiveFailures < backoffAfter || current >= max {
		return current
	}
	next := current * 2
	if next > max {
		return max
	}
	return next
}

type identityFile struct {
	NodeID           string                  `json:"nodeId"`
	NodeName         string                  `json:"nodeName"`
	Enrolled         bool                    `json:"enrolled"`
	Credential       string                  `json:"credential,omitempty"`
	TelemetryCursor  string                  `json:"telemetryCursor,omitempty"`
	PendingTelemetry *heartbeat.StatsRequest `json:"pendingTelemetry,omitempty"`
}

func collectAndSendStats(
	ctx context.Context,
	client heartbeat.Client,
	driver adguard.Driver,
	stateDir string,
	identity *identityFile,
) error {
	if identity.PendingTelemetry != nil {
		response, err := client.SendStats(ctx, *identity.PendingTelemetry, identity.Credential)
		if err != nil {
			return err
		}
		if response.BatchID != identity.PendingTelemetry.BatchID {
			return errors.New("central returned mismatched telemetry batch ID")
		}
		identity.TelemetryCursor = identity.PendingTelemetry.CollectedThrough
		identity.PendingTelemetry = nil
		return saveIdentity(stateDir, *identity)
	}
	since := time.Now().UTC().Add(-5 * time.Minute)
	if identity.TelemetryCursor != "" {
		parsed, err := time.Parse(time.RFC3339Nano, identity.TelemetryCursor)
		if err != nil {
			return fmt.Errorf("invalid telemetry cursor: %w", err)
		}
		since = parsed.UTC()
	}
	buckets, collectedThrough, err := driver.CollectStats(ctx, since)
	if err != nil {
		return err
	}
	if len(buckets) == 0 || !collectedThrough.After(since) {
		return nil
	}
	request := heartbeat.StatsRequest{
		NodeID:           identity.NodeID,
		CollectedThrough: collectedThrough.Format(time.RFC3339Nano),
		Buckets:          make([]heartbeat.StatBucket, 0, len(buckets)),
	}
	for _, bucket := range buckets {
		request.Buckets = append(request.Buckets, heartbeat.StatBucket{
			ClientID:      bucket.ClientID,
			BucketStart:   bucket.BucketStart,
			BucketSeconds: bucket.BucketSeconds,
			Queries:       bucket.Queries,
			Blocked:       bucket.Blocked,
			Cached:        bucket.Cached,
		})
	}
	payload, err := json.Marshal(request)
	if err != nil {
		return err
	}
	digest := sha256.Sum256(append([]byte(identity.TelemetryCursor+"|"), payload...))
	request.BatchID = fmt.Sprintf("stats-%x", digest[:16])
	identity.PendingTelemetry = &request
	if err := saveIdentity(stateDir, *identity); err != nil {
		identity.PendingTelemetry = nil
		return err
	}
	response, err := client.SendStats(ctx, request, identity.Credential)
	if err != nil {
		return err
	}
	if response.BatchID != request.BatchID {
		return errors.New("central returned mismatched telemetry batch ID")
	}
	identity.TelemetryCursor = collectedThrough.Format(time.RFC3339Nano)
	identity.PendingTelemetry = nil
	return saveIdentity(stateDir, *identity)
}

func loadOrCreateIdentity(stateDir string, nodeName string) (identityFile, error) {
	path := filepath.Join(stateDir, "identity.json")
	data, err := os.ReadFile(path)
	if err == nil {
		var identity identityFile
		if err := json.Unmarshal(data, &identity); err != nil {
			return identityFile{}, err
		}
		return identity, nil
	}
	if !os.IsNotExist(err) {
		return identityFile{}, err
	}
	hostname, _ := os.Hostname()
	identity := identityFile{
		NodeID:   stableNodeID(hostname),
		NodeName: nodeName,
	}
	return identity, saveIdentity(stateDir, identity)
}

func saveIdentity(stateDir string, identity identityFile) error {
	if err := os.MkdirAll(stateDir, 0700); err != nil {
		return err
	}
	data, err := json.MarshalIndent(identity, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(filepath.Join(stateDir, "identity.json"), data, 0600)
}

func stableNodeID(hostname string) string {
	hostname = strings.TrimSpace(hostname)
	if hostname == "" {
		hostname = "edge"
	}
	return "edge-" + strings.ToLower(strings.NewReplacer("_", "-", ".", "-").Replace(hostname))
}

func buildReport(ctx context.Context, cfg config.Config, nodeID string, driver adguard.Driver) heartbeat.NodeReport {
	hostname, _ := os.Hostname()
	osName := readFirstLine("/etc/os-release", "PRETTY_NAME")
	kernel := commandOutput("uname", "-r")
	unboundHealthy := commandOK("dig", "+time=3", "+tries=1", "@127.0.0.1", "-p", "5335", "example.com")
	adguardHealthy := driver.Healthy(ctx)
	health := "healthy"
	if !unboundHealthy || !adguardHealthy {
		health = "degraded"
	}
	return heartbeat.NodeReport{
		NodeID:         nodeID,
		NodeName:       cfg.NodeName,
		Hostname:       hostname,
		Architecture:   runtime.GOARCH,
		OS:             osName,
		Kernel:         kernel,
		SpeicheVersion: version,
		HealthStatus:   health,
		Inventory: map[string]any{
			"adguardHealthy": adguardHealthy,
			"unboundHealthy": unboundHealthy,
			"addresses":      localAddresses(),
			"publicServer":   false,
		},
	}
}

func readFirstLine(path string, key string) string {
	data, err := os.ReadFile(path)
	if err != nil {
		return "unknown"
	}
	for _, line := range strings.Split(string(data), "\n") {
		if strings.HasPrefix(line, key+"=") {
			return strings.Trim(strings.TrimPrefix(line, key+"="), `"`)
		}
	}
	return "unknown"
}

func commandOutput(name string, args ...string) string {
	out, err := exec.Command(name, args...).Output()
	if err != nil {
		return "unknown"
	}
	return strings.TrimSpace(string(out))
}

func commandOK(name string, args ...string) bool {
	return exec.Command(name, args...).Run() == nil
}

func localAddresses() []string {
	var addresses []string
	ifaces, err := net.Interfaces()
	if err != nil {
		return addresses
	}
	for _, iface := range ifaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}
		values, err := iface.Addrs()
		if err != nil {
			continue
		}
		for _, value := range values {
			addresses = append(addresses, value.String())
		}
	}
	return addresses
}
