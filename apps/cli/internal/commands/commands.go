package commands

import (
	"archive/tar"
	"compress/gzip"
	"errors"
	"flag"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	"codeberg.org/KyleHub/nabe/apps/cli/internal/config"
	"golang.org/x/crypto/bcrypt"
)

const version = "0.0.0"

func Run(args []string, cfg config.Config) error {
	if len(args) == 0 {
		return usage()
	}

	switch args[0] {
	case "version":
		fmt.Println("nabe cli " + version)
		return nil
	case "status":
		fmt.Printf("nabe api: %s\n", cfg.APIURL)
		return nil
	case "install":
		return runInstall(args[1:])
	default:
		return usage()
	}
}

func usage() error {
	return errors.New("usage: nabe <version|status|install --edge --remote <url> --token <token> [--adguard-ui-bind <addr:port>]>")
}

type edgeInstallOptions struct {
	Remote               string
	Token                string
	AdGuardUIBind        string
	AdGuardAdminUser     string
	AdGuardAdminPassword string
}

type hostFacts struct {
	OS           string
	Architecture string
	InitSystem   string
	PackageTool  string
	Addresses    []string
}

func runInstall(args []string) error {
	flags := flag.NewFlagSet("install", flag.ContinueOnError)
	flags.SetOutput(io.Discard)
	edge := flags.Bool("edge", false, "install as an edge DNS node")
	remote := flags.String("remote", "", "Nabe central API URL")
	token := flags.String("token", "", "dev edge token")
	adGuardUIBind := flags.String("adguard-ui-bind", "127.0.0.1:3000", "AdGuard Home UI/API bind address")
	adGuardAdminUser := flags.String("adguard-admin-user", "", "AdGuard Home admin username; required for non-loopback UI binds")
	adGuardAdminPassword := flags.String("adguard-admin-password", "", "AdGuard Home admin password; required for non-loopback UI binds")
	if err := flags.Parse(args); err != nil {
		return err
	}
	if !*edge {
		return errors.New("only edge installation is supported: pass --edge")
	}
	opts := edgeInstallOptions{
		Remote:               *remote,
		Token:                *token,
		AdGuardUIBind:        *adGuardUIBind,
		AdGuardAdminUser:     *adGuardAdminUser,
		AdGuardAdminPassword: *adGuardAdminPassword,
	}
	if err := validateEdgeOptions(opts); err != nil {
		return err
	}
	facts, err := detectHostFacts()
	if err != nil {
		return err
	}
	fmt.Printf("nabe edge bootstrap: os=%s arch=%s init=%s package=%s addresses=%s\n", facts.OS, facts.Architecture, facts.InitSystem, facts.PackageTool, strings.Join(facts.Addresses, ","))
	if facts.InitSystem != "systemd" {
		return fmt.Errorf("unsupported init system %q; systemd is required", facts.InitSystem)
	}
	if facts.PackageTool != "apt-get" {
		return fmt.Errorf("unsupported package tool %q; apt-get is required for this MVP bootstrap", facts.PackageTool)
	}

	steps := []struct {
		name string
		fn   func(edgeInstallOptions, hostFacts) error
	}{
		{"install runtime dependencies", installDependencies},
		{"create nabe directories", createNabeDirectories},
		{"configure unbound", configureUnbound},
		{"install adguard home", installAdGuardHome},
		{"install speiche", installSpeiche},
		{"write speiche service", configureSpeiche},
		{"run health checks", runHealthChecks},
	}
	for _, step := range steps {
		fmt.Println("==> " + step.name)
		if err := step.fn(opts, facts); err != nil {
			return fmt.Errorf("%s failed: %w", step.name, err)
		}
	}
	return nil
}

func validateEdgeOptions(opts edgeInstallOptions) error {
	if strings.TrimSpace(opts.AdGuardUIBind) == "" {
		opts.AdGuardUIBind = "127.0.0.1:3000"
	}
	if strings.TrimSpace(opts.Remote) == "" {
		return errors.New("--remote is required")
	}
	if strings.TrimSpace(opts.Token) == "" {
		return errors.New("--token is required")
	}
	parsed, err := url.Parse(opts.Remote)
	if err != nil || parsed.Scheme == "" || parsed.Host == "" {
		return errors.New("--remote must be an absolute URL")
	}
	host := strings.ToLower(parsed.Hostname())
	if host == "localhost" || host == "127.0.0.1" || host == "::1" {
		return errors.New("--remote must not point at localhost in edge mode")
	}
	if err := validateBindAddress(opts.AdGuardUIBind); err != nil {
		return fmt.Errorf("--adguard-ui-bind: %w", err)
	}
	if !isLoopbackBind(opts.AdGuardUIBind) {
		if strings.TrimSpace(opts.AdGuardAdminUser) == "" {
			return errors.New("--adguard-admin-user is required when --adguard-ui-bind is not loopback")
		}
		if strings.TrimSpace(opts.AdGuardAdminPassword) == "" {
			return errors.New("--adguard-admin-password is required when --adguard-ui-bind is not loopback")
		}
	}
	return nil
}

func validateBindAddress(value string) error {
	value = strings.TrimSpace(value)
	if value == "" {
		return errors.New("must not be empty")
	}
	host, port, err := net.SplitHostPort(value)
	if err != nil {
		return errors.New("must be in host:port form")
	}
	if strings.TrimSpace(host) == "" || strings.TrimSpace(port) == "" {
		return errors.New("must include host and port")
	}
	if net.ParseIP(host) == nil && host != "localhost" {
		return errors.New("host must be an IP address or localhost")
	}
	return nil
}

func isLoopbackBind(value string) bool {
	host, _, err := net.SplitHostPort(strings.TrimSpace(value))
	if err != nil {
		return false
	}
	if strings.EqualFold(host, "localhost") {
		return true
	}
	ip := net.ParseIP(host)
	return ip != nil && ip.IsLoopback()
}

func detectHostFacts() (hostFacts, error) {
	facts := hostFacts{
		OS:           osReleaseName(),
		Architecture: runtime.GOARCH,
		InitSystem:   "unknown",
		PackageTool:  "unknown",
		Addresses:    localAddresses(),
	}
	if _, err := exec.LookPath("systemctl"); err == nil {
		facts.InitSystem = "systemd"
	}
	if _, err := exec.LookPath("apt-get"); err == nil {
		facts.PackageTool = "apt-get"
	}
	return facts, nil
}

func installDependencies(edgeInstallOptions, hostFacts) error {
	if err := run("sudo", "apt-get", "update"); err != nil {
		return err
	}
	return run("sudo", "env", "DEBIAN_FRONTEND=noninteractive", "apt-get", "install", "-y",
		"ca-certificates", "curl", "dnsutils", "golang-go", "tar", "gzip", "unbound")
}

func createNabeDirectories(edgeInstallOptions, hostFacts) error {
	for _, dir := range []string{"/etc/nabe", "/var/lib/nabe/speiche", "/opt/nabe/src"} {
		if err := run("sudo", "install", "-d", "-m", "0750", dir); err != nil {
			return err
		}
	}
	return nil
}

func configureUnbound(edgeInstallOptions, hostFacts) error {
	const config = unboundConfig
	if err := writeRootFile("/etc/unbound/unbound.conf.d/nabe-edge.conf", config, "0644"); err != nil {
		return err
	}
	if err := run("sudo", "systemctl", "enable", "unbound"); err != nil {
		return err
	}
	return run("sudo", "systemctl", "restart", "unbound")
}

const unboundConfig = `server:
  interface: 127.0.0.1
  port: 5335
  access-control: 127.0.0.0/8 allow
  do-ip4: yes
  do-ip6: yes
  do-udp: yes
  do-tcp: yes
  hide-identity: yes
  hide-version: yes
  qname-minimisation: yes
`

func installAdGuardHome(opts edgeInstallOptions, facts hostFacts) error {
	if _, err := os.Stat("/opt/AdGuardHome/AdGuardHome"); os.IsNotExist(err) {
		tmp, err := os.MkdirTemp("", "nabe-adguard-*")
		if err != nil {
			return err
		}
		defer os.RemoveAll(tmp)
		arch := "arm64"
		if runtime.GOARCH == "amd64" {
			arch = "amd64"
		}
		archive := filepath.Join(tmp, "adguardhome.tar.gz")
		url := "https://static.adtidy.org/adguardhome/release/AdGuardHome_linux_" + arch + ".tar.gz"
		if err := download(url, archive); err != nil {
			return err
		}
		if err := untarGz(archive, tmp); err != nil {
			return err
		}
		if err := run("sudo", "rm", "-rf", "/opt/AdGuardHome"); err != nil {
			return err
		}
		if err := run("sudo", "mv", filepath.Join(tmp, "AdGuardHome"), "/opt/AdGuardHome"); err != nil {
			return err
		}
		if err := run("sudo", "/opt/AdGuardHome/AdGuardHome", "-s", "install"); err != nil {
			return err
		}
	}
	config, err := adGuardHomeConfig(opts)
	if err != nil {
		return err
	}
	if err := writeRootFile("/opt/AdGuardHome/AdGuardHome.yaml", config, "0600"); err != nil {
		return err
	}
	if err := run("sudo", "systemctl", "enable", "AdGuardHome"); err != nil {
		return err
	}
	return run("sudo", "systemctl", "restart", "AdGuardHome")
}

func adGuardHomeConfig(opts edgeInstallOptions) (string, error) {
	users := "users: []"
	if strings.TrimSpace(opts.AdGuardAdminUser) != "" || strings.TrimSpace(opts.AdGuardAdminPassword) != "" {
		hash, err := bcrypt.GenerateFromPassword([]byte(opts.AdGuardAdminPassword), bcrypt.DefaultCost)
		if err != nil {
			return "", err
		}
		users = fmt.Sprintf(`users:
  - name: %s
    password: %s`, yamlScalar(opts.AdGuardAdminUser), yamlScalar(string(hash)))
	}

	return fmt.Sprintf(`http:
  address: %s
%s
auth_attempts: 5
block_auth_min: 15
http_proxy: ""
language: en
theme: auto
dns:
  bind_hosts:
    - 0.0.0.0
  port: 53
  upstream_dns:
    - 127.0.0.1:5335
  bootstrap_dns:
    - 127.0.0.1:5335
  protection_enabled: true
  filtering_enabled: true
  blocking_mode: default
  ratelimit: 0
  cache_size: 4194304
filters: []
user_rules:
  - "||blocked.nabe.test^"
schema_version: 29
`, opts.AdGuardUIBind, users), nil
}

func installSpeiche(edgeInstallOptions, hostFacts) error {
	tmp, err := os.MkdirTemp("", "nabe-source-*")
	if err != nil {
		return err
	}
	defer os.RemoveAll(tmp)
	sourceURL := getenv("NABE_SOURCE_URL", "https://codeberg.org/KyleHub/nabe/archive/main.tar.gz")
	archive := filepath.Join(tmp, "nabe.tar.gz")
	if err := download(sourceURL, archive); err != nil {
		return err
	}
	if err := untarGz(archive, tmp); err != nil {
		return err
	}
	sourceRoot, err := findSourceRoot(tmp)
	if err != nil {
		return err
	}
	binary := filepath.Join(tmp, "speiche")
	cmd := exec.Command("go", "build", "-o", binary, ".")
	cmd.Dir = filepath.Join(sourceRoot, "apps", "speiche")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return err
	}
	return run("sudo", "install", "-m", "0755", binary, "/usr/local/bin/speiche")
}

func configureSpeiche(opts edgeInstallOptions, facts hostFacts) error {
	hostname, _ := os.Hostname()
	if hostname == "" {
		hostname = "nabe-edge"
	}
	env := fmt.Sprintf(`SPEICHE_NODE_NAME=%s
NABE_API_URL=%s
SPEICHE_ENROLLMENT_TOKEN=%s
ADGUARD_BASE_URL=http://127.0.0.1:3000
SPEICHE_STATE_DIR=/var/lib/nabe/speiche
SPEICHE_HEARTBEAT_INTERVAL_SECONDS=30
SPEICHE_HEARTBEAT_BACKOFF_AFTER_FAILURES=3
SPEICHE_HEARTBEAT_MAX_INTERVAL_SECONDS=300
`, shellValue(hostname), shellValue(opts.Remote), shellValue(opts.Token))
	if err := writeRootFile("/etc/nabe/speiche.env", env, "0600"); err != nil {
		return err
	}
	unit := `[Unit]
Description=Speiche Nabe Edge Agent
After=network-online.target AdGuardHome.service unbound.service
Wants=network-online.target

[Service]
EnvironmentFile=/etc/nabe/speiche.env
ExecStart=/usr/local/bin/speiche
Restart=always
RestartSec=10
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
`
	if err := writeRootFile("/etc/systemd/system/speiche.service", unit, "0644"); err != nil {
		return err
	}
	if err := run("sudo", "systemctl", "daemon-reload"); err != nil {
		return err
	}
	return run("sudo", "systemctl", "enable", "--now", "speiche")
}

func runHealthChecks(edgeInstallOptions, hostFacts) error {
	checks := [][]string{
		{"systemctl", "is-active", "unbound"},
		{"systemctl", "is-active", "AdGuardHome"},
		{"systemctl", "is-active", "speiche"},
		{"dig", "+time=3", "+tries=1", "@127.0.0.1", "-p", "5335", "example.com"},
		{"dig", "+time=3", "+tries=1", "@127.0.0.1", "example.com"},
	}
	for _, check := range checks {
		if err := run("sudo", check...); err != nil {
			return err
		}
	}
	out, err := exec.Command("dig", "+short", "+time=3", "+tries=1", "@127.0.0.1", "blocked.nabe.test", "A").CombinedOutput()
	if err != nil {
		return fmt.Errorf("blocked-domain query failed: %w: %s", err, strings.TrimSpace(string(out)))
	}
	if !strings.Contains(string(out), "0.0.0.0") {
		return fmt.Errorf("blocked-domain query was not blocked; got %q", strings.TrimSpace(string(out)))
	}
	return nil
}

func run(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	return cmd.Run()
}

func writeRootFile(path string, content string, mode string) error {
	tmp, err := os.CreateTemp("", "nabe-root-file-*")
	if err != nil {
		return err
	}
	tmpPath := tmp.Name()
	defer os.Remove(tmpPath)
	if _, err := tmp.WriteString(content); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	if err := run("sudo", "install", "-m", mode, tmpPath, path); err != nil {
		return err
	}
	return nil
}

func download(sourceURL string, path string) error {
	client := http.Client{Timeout: 5 * time.Minute}
	resp, err := client.Get(sourceURL)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("download %s returned HTTP %d", sourceURL, resp.StatusCode)
	}
	file, err := os.Create(path)
	if err != nil {
		return err
	}
	defer file.Close()
	_, err = io.Copy(file, resp.Body)
	return err
}

func untarGz(archive string, dest string) error {
	file, err := os.Open(archive)
	if err != nil {
		return err
	}
	defer file.Close()
	gz, err := gzip.NewReader(file)
	if err != nil {
		return err
	}
	defer gz.Close()
	tr := tar.NewReader(gz)
	for {
		header, err := tr.Next()
		if errors.Is(err, io.EOF) {
			return nil
		}
		if err != nil {
			return err
		}
		target := filepath.Join(dest, header.Name)
		cleanDest, _ := filepath.Abs(dest)
		cleanTarget, _ := filepath.Abs(target)
		if !strings.HasPrefix(cleanTarget, cleanDest+string(os.PathSeparator)) {
			return fmt.Errorf("archive path escapes destination: %s", header.Name)
		}
		switch header.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(target, 0755); err != nil {
				return err
			}
		case tar.TypeReg:
			if err := os.MkdirAll(filepath.Dir(target), 0755); err != nil {
				return err
			}
			out, err := os.OpenFile(target, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.FileMode(header.Mode))
			if err != nil {
				return err
			}
			if _, err := io.Copy(out, tr); err != nil {
				out.Close()
				return err
			}
			if err := out.Close(); err != nil {
				return err
			}
		}
	}
}

func findSourceRoot(tmp string) (string, error) {
	matches, err := filepath.Glob(filepath.Join(tmp, "*", "apps", "speiche", "go.mod"))
	if err != nil {
		return "", err
	}
	if len(matches) == 0 {
		return "", errors.New("downloaded Nabe source did not contain apps/speiche/go.mod")
	}
	return filepath.Dir(filepath.Dir(filepath.Dir(matches[0]))), nil
}

func osReleaseName() string {
	data, err := os.ReadFile("/etc/os-release")
	if err != nil {
		return "unknown"
	}
	for _, line := range strings.Split(string(data), "\n") {
		if strings.HasPrefix(line, "PRETTY_NAME=") {
			return strings.Trim(strings.TrimPrefix(line, "PRETTY_NAME="), `"`)
		}
	}
	return "unknown"
}

func localAddresses() []string {
	var values []string
	ifaces, err := net.Interfaces()
	if err != nil {
		return values
	}
	for _, iface := range ifaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}
		addrs, err := iface.Addrs()
		if err != nil {
			continue
		}
		for _, addr := range addrs {
			values = append(values, addr.String())
		}
	}
	return values
}

func shellValue(value string) string {
	return "'" + strings.ReplaceAll(value, "'", "'\"'\"'") + "'"
}

func yamlScalar(value string) string {
	return strconv.Quote(value)
}

func getenv(key string, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}
	return value
}
