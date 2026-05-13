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
	if !strings.Contains(adGuardHomeConfig("127.0.0.1:3000"), "address: 127.0.0.1:3000") {
		t.Fatalf("AdGuard Home UI must bind to loopback only")
	}
}

func TestAdGuardHomeConfigAllowsExplicitPublicUIBind(t *testing.T) {
	if !strings.Contains(adGuardHomeConfig("0.0.0.0:3000"), "address: 0.0.0.0:3000") {
		t.Fatalf("AdGuard Home UI should use explicit bind address")
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
