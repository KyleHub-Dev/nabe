package adguard

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestCollectStatsAggregatesExactClientIDs(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/control/querylog" || r.URL.Query().Get("limit") != "5000" {
			t.Fatalf("unexpected request %s", r.URL.String())
		}
		username, password, ok := r.BasicAuth()
		if !ok || username != "admin" || password != "secret" {
			t.Fatal("missing basic authentication")
		}
		_ = json.NewEncoder(w).Encode(queryLogResponse{Data: []queryLogEntry{
			{ClientID: "phone", Time: "2026-07-12T12:04:00Z", Reason: "NotFilteredNotFound", Cached: true},
			{ClientID: "phone", Time: "2026-07-12T12:03:00Z", Reason: "FilteredBlackList"},
			{ClientID: "other", Time: "2026-07-12T11:59:00Z", Reason: "FilteredBlackList"},
			{ClientID: "", Time: "2026-07-12T12:04:30Z", Reason: "NotFilteredNotFound"},
		}})
	}))
	defer server.Close()

	driver := NewDriver(server.URL, "admin", "secret")
	since, _ := time.Parse(time.RFC3339, "2026-07-12T12:00:00Z")
	buckets, through, err := driver.CollectStats(context.Background(), since)
	if err != nil {
		t.Fatalf("collect: %v", err)
	}
	if len(buckets) != 1 {
		t.Fatalf("expected one bucket, got %#v", buckets)
	}
	if buckets[0].ClientID != "phone" || buckets[0].Queries != 2 || buckets[0].Blocked != 1 || buckets[0].Cached != 1 {
		t.Fatalf("unexpected bucket %#v", buckets[0])
	}
	if through.Format(time.RFC3339) != "2026-07-12T12:04:00Z" {
		t.Fatalf("unexpected cursor %s", through)
	}
}
