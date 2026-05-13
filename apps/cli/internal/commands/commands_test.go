package commands

import "testing"

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
