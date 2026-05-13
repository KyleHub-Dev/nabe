package heartbeat

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHeartbeatSendsBearerToken(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("authorization") != "Bearer token" {
			t.Fatalf("unexpected authorization header %q", r.Header.Get("authorization"))
		}
		_ = json.NewEncoder(w).Encode(NodeResponse{NodeID: "edge-pi", HealthStatus: "healthy"})
	}))
	defer server.Close()

	client := NewClient(server.URL, "token")
	_, err := client.Heartbeat(context.Background(), NodeReport{
		NodeID:         "edge-pi",
		NodeName:       "pi",
		Hostname:       "pi",
		Architecture:   "arm64",
		OS:             "linux",
		Kernel:         "test",
		SpeicheVersion: "test",
		HealthStatus:   "healthy",
		Inventory:      map[string]any{},
	})
	if err != nil {
		t.Fatalf("heartbeat failed: %v", err)
	}
}
