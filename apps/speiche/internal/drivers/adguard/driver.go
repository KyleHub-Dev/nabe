package adguard

import (
	"context"
	"net/http"
	"strings"
	"time"
)

type Driver struct {
	baseURL string
}

func NewDriver(baseURL string) Driver {
	return Driver{baseURL: baseURL}
}

func (d Driver) Status() string {
	return "local AdGuard driver placeholder: " + d.baseURL
}

func (d Driver) Healthy(ctx context.Context) bool {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, strings.TrimRight(d.baseURL, "/")+"/control/status", nil)
	if err != nil {
		return false
	}
	client := http.Client{Timeout: 3 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	return resp.StatusCode >= 200 && resp.StatusCode < 500
}
