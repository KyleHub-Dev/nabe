package main

import (
	"context"
	"encoding/json"
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

	"codeberg.org/KyleHub/nabe/apps/speiche/internal/config"
	"codeberg.org/KyleHub/nabe/apps/speiche/internal/drivers/adguard"
	"codeberg.org/KyleHub/nabe/apps/speiche/internal/heartbeat"
)

const version = "0.0.0"

func main() {
	cfg := config.FromEnv()
	client := heartbeat.NewClient(cfg.NabeAPIURL, cfg.EnrollmentToken)
	driver := adguard.NewDriver(cfg.AdGuardURL)
	interval, err := strconv.Atoi(cfg.IntervalSeconds)
	if err != nil || interval < 5 {
		interval = 30
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

	for {
		ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
		report := buildReport(ctx, cfg, identity.NodeID, driver)
		var response heartbeat.NodeResponse
		if !identity.Enrolled {
			response, err = client.Enroll(ctx, report)
			if err == nil {
				identity.Enrolled = true
				err = saveIdentity(cfg.StateDir, identity)
			}
		} else {
			response, err = client.Heartbeat(ctx, report)
		}
		cancel()
		if err != nil {
			log.Printf("heartbeat failed: %v", err)
		} else {
			log.Printf("heartbeat ok: node=%s status=%s last_seen=%s", response.NodeID, response.HealthStatus, response.LastSeenAt)
		}
		time.Sleep(time.Duration(interval) * time.Second)
	}
}

type identityFile struct {
	NodeID   string `json:"nodeId"`
	NodeName string `json:"nodeName"`
	Enrolled bool   `json:"enrolled"`
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
