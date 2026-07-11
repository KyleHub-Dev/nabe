package commands

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"gopkg.in/yaml.v3"
)

func TestValidateEdgeOptionsRejectsLocalhost(t *testing.T) {
	cases := []string{
		"http://localhost:8080",
		"http://127.0.0.1:8080",
		"http://[::1]:8080",
	}
	for _, remote := range cases {
		if err := validateEdgeOptions(edgeInstallOptions{Remote: remote, Token: "token"}); err == nil {
			t.Fatalf("expected %s to be rejected", remote)
		}
	}
}

func TestValidateEdgeOptionsAcceptsLanRemote(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		Remote: "http://10.0.0.230:8080",
		Token:  "token",
	})
	if err != nil {
		t.Fatalf("expected LAN remote to be accepted: %v", err)
	}
}

func TestValidateEdgeOptionsAllowsDeferredEnrollment(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		DeferEnrollment: true,
	})
	if err != nil {
		t.Fatalf("expected deferred enrollment to be accepted: %v", err)
	}
}

func TestValidateEdgeOptionsRejectsDeferredEnrollmentWithRemote(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		Remote:          "http://10.0.0.230:8080",
		DeferEnrollment: true,
	})
	if err == nil {
		t.Fatalf("expected deferred enrollment with remote to be rejected")
	}
}

func TestValidateEdgeOptionsAllowsReuseEnrollmentWithNewRemote(t *testing.T) {
	err := validateEdgeOptionSyntax(edgeInstallOptions{
		Remote:          "http://nabe.local:8080",
		ReuseEnrollment: true,
	})
	if err != nil {
		t.Fatalf("expected enrollment reuse to accept a replacement remote: %v", err)
	}
}

func TestValidateEdgeOptionsRejectsReuseEnrollmentWithToken(t *testing.T) {
	err := validateEdgeOptionSyntax(edgeInstallOptions{
		Token:           "replacement-token",
		ReuseEnrollment: true,
	})
	if err == nil {
		t.Fatal("expected enrollment reuse with an explicit token to be rejected")
	}
}

func TestUnboundConfigUsesExplicitPort(t *testing.T) {
	config := unboundConfig(false)
	if !strings.Contains(config, "interface: 127.0.0.1\n  port: 5335") {
		t.Fatalf("unbound config must bind the local resolver on 127.0.0.1:5335")
	}
}

func TestUnboundConfigUsesResilientStrictDefaults(t *testing.T) {
	config := unboundConfig(false)
	for _, want := range []string{
		"do-ip6: no",
		"edns-buffer-size: 1232",
		"prefetch: yes",
		"prefetch-key: yes",
		`module-config: "validator iterator"`,
		"serve-expired: yes",
		"serve-expired-ttl: 86400",
		"serve-expired-client-timeout: 1800",
		"val-permissive-mode: no",
		"log-servfail: yes",
	} {
		if !strings.Contains(config, want) {
			t.Fatalf("unbound config missing %q", want)
		}
	}
}

func TestUnboundConfigEnablesIPv6OnlyWithDefaultRoute(t *testing.T) {
	if !strings.Contains(unboundConfig(true), "do-ip6: yes") {
		t.Fatal("expected IPv6 recursion with an IPv6 default route")
	}
}

func TestAdGuardHomeConfigBindsUIToLoopback(t *testing.T) {
	config, err := adGuardHomeConfig(edgeInstallOptions{AdGuardUIBind: "127.0.0.1:3000"})
	if err != nil {
		t.Fatalf("config failed: %v", err)
	}
	if !strings.Contains(config, "address: 127.0.0.1:3000") {
		t.Fatalf("AdGuard Home UI must bind to loopback only")
	}
	if !strings.Contains(config, "users: []") {
		t.Fatalf("loopback-only AdGuard Home UI may omit users for local dev")
	}
	for _, want := range []string{
		"upstream_dns:\n    - 127.0.0.1:5335",
		"fallback_dns:\n    - https://1.1.1.1/dns-query",
		"upstream_timeout: 5s",
		"enable_dnssec: true",
		"ratelimit: 100",
		"cache_optimistic: true",
		"allowed_clients:",
		"pending_requests:\n    enabled: true",
		"querylog:\n  enabled: true",
		"interval: 168h",
		"url: https://adguardteam.github.io/HostlistsRegistry/assets/filter_1.txt",
	} {
		if !strings.Contains(config, want) {
			t.Fatalf("AdGuard Home config missing %q", want)
		}
	}
}

func TestParseGeneratedEnv(t *testing.T) {
	values := parseGeneratedEnv("ADGUARD_UI_ALIAS='adguard.home'\nADGUARD_ADMIN_PASSWORD='secret'\n")
	if values["ADGUARD_UI_ALIAS"] != "adguard.home" || values["ADGUARD_ADMIN_PASSWORD"] != "secret" {
		t.Fatalf("unexpected parsed values: %#v", values)
	}
}

func TestPreserveExistingDHCPShape(t *testing.T) {
	var snapshot adGuardConfigSnapshot
	err := yaml.Unmarshal([]byte(`dhcp:
  enabled: true
  interface_name: eth0
  dhcpv4:
    gateway_ip: 10.0.0.1
    subnet_mask: 255.255.255.0
    range_start: 10.0.0.100
    range_end: 10.0.0.250
`), &snapshot)
	if err != nil {
		t.Fatalf("parse DHCP snapshot: %v", err)
	}
	if !snapshot.DHCP.Enabled || snapshot.DHCP.DHCPv4.RangeStart != "10.0.0.100" {
		t.Fatalf("unexpected DHCP snapshot: %#v", snapshot)
	}
}

func TestAdGuardHomeConfigAcceptedByBinary(t *testing.T) {
	binary := os.Getenv("NABE_ADGUARD_BINARY")
	if binary == "" {
		t.Skip("NABE_ADGUARD_BINARY is not set")
	}
	config, err := adGuardHomeConfig(edgeInstallOptions{AdGuardUIBind: "127.0.0.1:3000"})
	if err != nil {
		t.Fatalf("config failed: %v", err)
	}
	tmp := t.TempDir()
	path := filepath.Join(tmp, "AdGuardHome.yaml")
	if err := os.WriteFile(path, []byte(config), 0600); err != nil {
		t.Fatalf("write config: %v", err)
	}
	cmd := exec.Command(binary, "--check-config", "-c", path, "-w", tmp)
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("AdGuard Home rejected generated config: %v: %s", err, strings.TrimSpace(string(out)))
	}
}

func TestAdGuardHomeConfigAllowsExplicitPublicUIBind(t *testing.T) {
	config, err := adGuardHomeConfig(edgeInstallOptions{
		AdGuardUIBind:        "0.0.0.0:3000",
		AdGuardAdminUser:     "admin",
		AdGuardAdminPassword: "secret",
	})
	if err != nil {
		t.Fatalf("config failed: %v", err)
	}
	if !strings.Contains(config, "address: 0.0.0.0:3000") {
		t.Fatalf("AdGuard Home UI should use explicit bind address")
	}
	if !strings.Contains(config, "name: \"admin\"") || !strings.Contains(config, "password: \"$2") {
		t.Fatalf("public AdGuard Home UI should include bcrypt-backed admin user")
	}
}

func TestValidateEdgeOptionsRejectsInvalidUIBind(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		Remote:        "http://10.0.0.230:8080",
		Token:         "token",
		AdGuardUIBind: "3000",
	})
	if err == nil {
		t.Fatalf("expected invalid UI bind to be rejected")
	}
}

func TestValidateEdgeOptionsRequiresAdminForPublicUIBind(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		Remote:        "http://10.0.0.230:8080",
		Token:         "token",
		AdGuardUIBind: "0.0.0.0:3000",
	})
	if err == nil {
		t.Fatalf("expected public UI bind to require admin credentials")
	}
}

func TestValidateEdgeOptionsRequiresDHCPSettingsWhenEnabled(t *testing.T) {
	err := validateEdgeOptions(edgeInstallOptions{
		Remote:              "http://10.0.0.230:8080",
		Token:               "token",
		AdGuardDHCPEnabled:  true,
		AdGuardDHCPSubnet:   "255.255.255.0",
		AdGuardDHCPGateway:  "10.0.0.1",
		AdGuardDHCPRangeEnd: "10.0.0.250",
	})
	if err == nil {
		t.Fatalf("expected incomplete DHCP settings to be rejected")
	}
}

func TestAdGuardHomeConfigCanEnableDHCP(t *testing.T) {
	config, err := adGuardHomeConfig(edgeInstallOptions{
		AdGuardUIBind:         "127.0.0.1:3000",
		AdGuardDHCPEnabled:    true,
		AdGuardDHCPInterface:  "eth0",
		AdGuardDHCPGateway:    "10.0.0.1",
		AdGuardDHCPSubnet:     "255.255.255.0",
		AdGuardDHCPRangeStart: "10.0.0.100",
		AdGuardDHCPRangeEnd:   "10.0.0.250",
	})
	if err != nil {
		t.Fatalf("config failed: %v", err)
	}
	for _, want := range []string{
		"dhcp:",
		"enabled: true",
		"interface_name: \"eth0\"",
		"gateway_ip: \"10.0.0.1\"",
		"range_start: \"10.0.0.100\"",
		"range_end: \"10.0.0.250\"",
	} {
		if !strings.Contains(config, want) {
			t.Fatalf("config missing %q", want)
		}
	}
}

func TestPrepareEdgeOptionsPublishesExplicitAdGuardAlias(t *testing.T) {
	opts, err := prepareEdgeOptions(edgeInstallOptions{
		AdGuardUIBind:  "127.0.0.1:3000",
		AdGuardUIAlias: "adguard.home",
	}, hostFacts{Addresses: []string{"127.0.0.1/8", "10.0.0.10/24"}})
	if err != nil {
		t.Fatalf("prepare failed: %v", err)
	}
	if opts.AdGuardUIBind != "10.0.0.10:3000" {
		t.Fatalf("expected UI bind to use detected LAN IP, got %q", opts.AdGuardUIBind)
	}
	if opts.AdGuardAdminUser == "" || opts.AdGuardAdminPassword == "" {
		t.Fatalf("expected generated break-glass credentials")
	}
}

func TestPrepareEdgeOptionsGeneratesCredentialsForExplicitAliasBind(t *testing.T) {
	opts, err := prepareEdgeOptions(edgeInstallOptions{
		Remote:           "http://10.0.0.230:8080",
		Token:            "dev-token",
		AdGuardUIBind:    "10.0.0.10:3000",
		AdGuardUIAlias:   "adguard.home",
		AdGuardUIAliasIP: "10.0.0.10",
	}, hostFacts{Addresses: []string{"10.0.0.10/24"}})
	if err != nil {
		t.Fatalf("prepare failed: %v", err)
	}
	if err := validateEdgeOptions(opts); err != nil {
		t.Fatalf("prepared options should validate: %v", err)
	}
	if opts.AdGuardAdminUser == "" || opts.AdGuardAdminPassword == "" {
		t.Fatalf("expected generated break-glass credentials")
	}
}

func TestAdGuardHomeConfigAddsAliasDnsRewrite(t *testing.T) {
	config, err := adGuardHomeConfig(edgeInstallOptions{
		AdGuardUIBind:        "10.0.0.10:3000",
		AdGuardUIAlias:       "adguard.home",
		AdGuardUIAliasIP:     "10.0.0.10",
		AdGuardAdminUser:     "nabe-admin",
		AdGuardAdminPassword: "secret",
	})
	if err != nil {
		t.Fatalf("config failed: %v", err)
	}
	if !strings.Contains(config, `||adguard.home^$dnsrewrite=NOERROR;A;10.0.0.10`) {
		t.Fatalf("expected adguard.home DNS rewrite in config")
	}
}
