package commands

import (
	"strings"
	"testing"
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

func TestUnboundConfigUsesExplicitPort(t *testing.T) {
	if !strings.Contains(unboundConfig, "interface: 127.0.0.1\n  port: 5335") {
		t.Fatalf("unbound config must bind the local resolver on 127.0.0.1:5335")
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
