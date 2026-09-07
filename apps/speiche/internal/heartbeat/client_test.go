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
	}, "token")
	if err != nil {
		t.Fatalf("heartbeat failed: %v", err)
	}
}

func TestSendStatsSendsIdempotencyBatch(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/edge/stats" || r.Header.Get("authorization") != "Bearer token" {
			t.Fatalf("unexpected request %s", r.URL.Path)
		}
		var request StatsRequest
		if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
			t.Fatal(err)
		}
		_ = json.NewEncoder(w).Encode(StatsResponse{Accepted: true, BatchID: request.BatchID, BucketCount: len(request.Buckets)})
	}))
	defer server.Close()
	client := NewClient(server.URL, "token")
	response, err := client.SendStats(context.Background(), StatsRequest{
		BatchID: "batch-1", NodeID: "edge-pi", CollectedThrough: "2026-07-12T12:05:00Z",
		Buckets: []StatBucket{{ClientID: "phone", BucketStart: "2026-07-12T12:00:00Z", BucketSeconds: 300, Queries: 1}},
	}, "token")
	if err != nil || response.BatchID != "batch-1" {
		t.Fatalf("stats upload failed: %#v %v", response, err)
	}
}

func TestForbiddenCredentialIsTyped(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "forbidden", http.StatusForbidden)
	}))
	defer server.Close()
	client := NewClient(server.URL, "bootstrap")
	_, err := client.Heartbeat(context.Background(), NodeReport{NodeID: "edge-pi"}, "revoked")
	if !IsForbidden(err) {
		t.Fatalf("expected typed forbidden error, got %v", err)
	}
}
