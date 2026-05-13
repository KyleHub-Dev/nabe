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
}

func (c Client) Enroll(ctx context.Context, report NodeReport) (NodeResponse, error) {
	return c.post(ctx, "/api/edge/enroll", report)
}

func (c Client) Heartbeat(ctx context.Context, report NodeReport) (NodeResponse, error) {
	return c.post(ctx, "/api/edge/heartbeat", report)
}

func (c Client) post(ctx context.Context, path string, report NodeReport) (NodeResponse, error) {
	if c.enrollmentToken == "" {
		return NodeResponse{}, errors.New("SPEICHE_ENROLLMENT_TOKEN is required")
	}
	body, err := json.Marshal(report)
	if err != nil {
		return NodeResponse{}, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.apiURL+path, bytes.NewReader(body))
	if err != nil {
		return NodeResponse{}, err
	}
	req.Header.Set("authorization", "Bearer "+c.enrollmentToken)
	req.Header.Set("content-type", "application/json")

	resp, err := c.http.Do(req)
	if err != nil {
		return NodeResponse{}, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return NodeResponse{}, fmt.Errorf("central returned HTTP %d", resp.StatusCode)
	}
	var response NodeResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return NodeResponse{}, err
	}
	return response, nil
}
