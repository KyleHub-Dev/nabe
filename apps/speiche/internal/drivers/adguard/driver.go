package adguard

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"time"
)

const (
	bucketDuration = 5 * time.Minute
	pageLimit      = 5000
	maxPages       = 20
)

type Driver struct {
	baseURL  string
	username string
	password string
	http     http.Client
}

type StatBucket struct {
	ClientID      string `json:"clientId"`
	BucketStart   string `json:"bucketStart"`
	BucketSeconds int64  `json:"bucketSeconds"`
	Queries       uint64 `json:"queries"`
	Blocked       uint64 `json:"blocked"`
	Cached        uint64 `json:"cached"`
}

type queryLogResponse struct {
	Data []queryLogEntry `json:"data"`
}

type queryLogEntry struct {
	ClientID string `json:"client_id"`
	Time     string `json:"time"`
	Reason   string `json:"reason"`
	Cached   bool   `json:"cached"`
}

func NewDriver(baseURL string, username string, password string) Driver {
	return Driver{
		baseURL:  strings.TrimRight(baseURL, "/"),
		username: username,
		password: password,
		http:     http.Client{Timeout: 10 * time.Second},
	}
}

func (d Driver) Status() string {
	return "local AdGuard driver ready: " + d.baseURL
}

func (d Driver) Healthy(ctx context.Context) bool {
	req, err := d.request(ctx, "/control/status")
	if err != nil {
		return false
	}
	resp, err := d.http.Do(req)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	return resp.StatusCode >= 200 && resp.StatusCode < 300
}

func (d Driver) CollectStats(ctx context.Context, since time.Time) ([]StatBucket, time.Time, error) {
	if since.IsZero() {
		since = time.Now().UTC().Add(-bucketDuration)
	}
	type key struct {
		clientID string
		start    time.Time
	}
	counts := make(map[key]*StatBucket)
	collectedThrough := since
	olderThan := ""
	for page := 0; page < maxPages; page++ {
		response, err := d.queryLogPage(ctx, olderThan)
		if err != nil {
			return nil, since, err
		}
		oldest := time.Time{}
		for _, entry := range response.Data {
			timestamp, err := time.Parse(time.RFC3339Nano, entry.Time)
			if err != nil || entry.ClientID == "" {
				continue
			}
			timestamp = timestamp.UTC()
			if oldest.IsZero() || timestamp.Before(oldest) {
				oldest = timestamp
			}
			if !timestamp.After(since) {
				continue
			}
			if timestamp.After(collectedThrough) {
				collectedThrough = timestamp
			}
			start := timestamp.Truncate(bucketDuration)
			bucketKey := key{clientID: entry.ClientID, start: start}
			bucket := counts[bucketKey]
			if bucket == nil {
				bucket = &StatBucket{
					ClientID:      entry.ClientID,
					BucketStart:   start.Format(time.RFC3339),
					BucketSeconds: int64(bucketDuration.Seconds()),
				}
				counts[bucketKey] = bucket
			}
			bucket.Queries++
			if strings.HasPrefix(entry.Reason, "Filtered") {
				bucket.Blocked++
			}
			if entry.Cached {
				bucket.Cached++
			}
		}
		if len(response.Data) < pageLimit || oldest.IsZero() || !oldest.After(since) {
			buckets := make([]StatBucket, 0, len(counts))
			for _, bucket := range counts {
				buckets = append(buckets, *bucket)
			}
			sort.Slice(buckets, func(i, j int) bool {
				if buckets[i].BucketStart == buckets[j].BucketStart {
					return buckets[i].ClientID < buckets[j].ClientID
				}
				return buckets[i].BucketStart < buckets[j].BucketStart
			})
			if len(buckets) > 1000 {
				return nil, since, errors.New("telemetry collection exceeded 1000 buckets")
			}
			return buckets, collectedThrough, nil
		}
		olderThan = oldest.Format(time.RFC3339Nano)
	}
	return nil, since, errors.New("telemetry collection exceeded pagination safety limit")
}

func (d Driver) queryLogPage(ctx context.Context, olderThan string) (queryLogResponse, error) {
	endpoint, err := url.Parse(d.baseURL + "/control/querylog")
	if err != nil {
		return queryLogResponse{}, err
	}
	query := endpoint.Query()
	query.Set("limit", fmt.Sprintf("%d", pageLimit))
	if olderThan != "" {
		query.Set("older_than", olderThan)
	}
	endpoint.RawQuery = query.Encode()
	req, err := d.request(ctx, endpoint.String())
	if err != nil {
		return queryLogResponse{}, err
	}
	resp, err := d.http.Do(req)
	if err != nil {
		return queryLogResponse{}, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return queryLogResponse{}, fmt.Errorf("AdGuard query log returned HTTP %d", resp.StatusCode)
	}
	var response queryLogResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return queryLogResponse{}, err
	}
	return response, nil
}

func (d Driver) request(ctx context.Context, path string) (*http.Request, error) {
	endpoint := path
	if strings.HasPrefix(path, "/") {
		endpoint = d.baseURL + path
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, endpoint, nil)
	if err != nil {
		return nil, err
	}
	if d.username != "" || d.password != "" {
		req.SetBasicAuth(d.username, d.password)
	}
	return req, nil
}
