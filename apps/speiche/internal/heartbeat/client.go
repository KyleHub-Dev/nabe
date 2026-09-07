package heartbeat

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"
)

type Client struct {
	apiURL          string
	enrollmentToken string
	http            http.Client
}

type HTTPStatusError struct {
	Endpoint string
	Status   int
}

func (e *HTTPStatusError) Error() string {
	return fmt.Sprintf("central endpoint %s returned HTTP %d", e.Endpoint, e.Status)
}

func IsForbidden(err error) bool {
	var statusError *HTTPStatusError
	return errors.As(err, &statusError) && statusError.Status == http.StatusForbidden
}

func NewClient(apiURL string, enrollmentToken string) Client {
	return Client{
		apiURL:          strings.TrimRight(apiURL, "/"),
		enrollmentToken: enrollmentToken,
		http:            http.Client{Timeout: 10 * time.Second},
	}
}

func (c Client) Status() string {
	if c.enrollmentToken == "" {
		return "heartbeat client ready without enrollment token"
	}

	return "heartbeat client ready for " + c.apiURL
}

type NodeReport struct {
	NodeID         string         `json:"nodeId"`
	NodeName       string         `json:"nodeName"`
	Hostname       string         `json:"hostname"`
	Architecture   string         `json:"architecture"`
	OS             string         `json:"os"`
	Kernel         string         `json:"kernel"`
	SpeicheVersion string         `json:"speicheVersion"`
	HealthStatus   string         `json:"healthStatus"`
	Inventory      map[string]any `json:"inventory"`
}

type NodeResponse struct {
	NodeID       string `json:"nodeId"`
	NodeName     string `json:"nodeName"`
	HealthStatus string `json:"healthStatus"`
	LastSeenAt   string `json:"lastSeenAt"`
	Credential   string `json:"credential,omitempty"`
}

type StatBucket struct {
	ClientID      string `json:"clientId"`
	BucketStart   string `json:"bucketStart"`
	BucketSeconds int64  `json:"bucketSeconds"`
	Queries       uint64 `json:"queries"`
	Blocked       uint64 `json:"blocked"`
	Cached        uint64 `json:"cached"`
}

type StatsRequest struct {
	BatchID          string       `json:"batchId"`
	NodeID           string       `json:"nodeId"`
	CollectedThrough string       `json:"collectedThrough"`
	Buckets          []StatBucket `json:"buckets"`
}

type StatsResponse struct {
	Accepted    bool   `json:"accepted"`
	BatchID     string `json:"batchId"`
	BucketCount int    `json:"bucketCount"`
}

func (c Client) Enroll(ctx context.Context, report NodeReport) (NodeResponse, error) {
	return c.post(ctx, "/api/edge/enroll", report)
}

func (c Client) Heartbeat(ctx context.Context, report NodeReport, credential string) (NodeResponse, error) {
	return c.postWithToken(ctx, "/api/edge/heartbeat", report, credential)
}

func (c Client) SendStats(ctx context.Context, request StatsRequest, credential string) (StatsResponse, error) {
	if credential == "" {
		return StatsResponse{}, errors.New("edge credential is required")
	}
	body, err := json.Marshal(request)
	if err != nil {
		return StatsResponse{}, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.apiURL+"/api/edge/stats", bytes.NewReader(body))
	if err != nil {
		return StatsResponse{}, err
	}
	req.Header.Set("authorization", "Bearer "+credential)
	req.Header.Set("content-type", "application/json")
	resp, err := c.http.Do(req)
	if err != nil {
		return StatsResponse{}, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return StatsResponse{}, &HTTPStatusError{Endpoint: "/api/edge/stats", Status: resp.StatusCode}
	}
	var response StatsResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return StatsResponse{}, err
	}
	return response, nil
}

func (c Client) post(ctx context.Context, path string, report NodeReport) (NodeResponse, error) {
	if c.enrollmentToken == "" {
		return NodeResponse{}, errors.New("SPEICHE_ENROLLMENT_TOKEN is required")
	}
	return c.postWithToken(ctx, path, report, c.enrollmentToken)
}

func (c Client) postWithToken(ctx context.Context, path string, report NodeReport, token string) (NodeResponse, error) {
	if token == "" {
		return NodeResponse{}, errors.New("edge credential is required")
	}
	body, err := json.Marshal(report)
	if err != nil {
		return NodeResponse{}, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.apiURL+path, bytes.NewReader(body))
	if err != nil {
		return NodeResponse{}, err
	}
	req.Header.Set("authorization", "Bearer "+token)
	req.Header.Set("content-type", "application/json")

	resp, err := c.http.Do(req)
	if err != nil {
		return NodeResponse{}, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return NodeResponse{}, &HTTPStatusError{Endpoint: path, Status: resp.StatusCode}
	}
	var response NodeResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return NodeResponse{}, err
	}
	return response, nil
}
