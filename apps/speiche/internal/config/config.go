package config

import "os"

type Config struct {
	NodeName            string
	NabeAPIURL          string
	EnrollmentToken     string
	AdGuardURL          string
	StateDir            string
	IntervalSeconds     string
	MaxIntervalSeconds  string
	FailureBackoffAfter string
}

func FromEnv() Config {
	return Config{
		NodeName:            getenv("SPEICHE_NODE_NAME", "speiche-local"),
		NabeAPIURL:          getenv("NABE_API_URL", "http://localhost:8080"),
		EnrollmentToken:     os.Getenv("SPEICHE_ENROLLMENT_TOKEN"),
		AdGuardURL:          getenv("ADGUARD_BASE_URL", "http://127.0.0.1:3000"),
		StateDir:            getenv("SPEICHE_STATE_DIR", "/var/lib/nabe/speiche"),
		IntervalSeconds:     getenv("SPEICHE_HEARTBEAT_INTERVAL_SECONDS", "30"),
		MaxIntervalSeconds:  getenv("SPEICHE_HEARTBEAT_MAX_INTERVAL_SECONDS", "300"),
		FailureBackoffAfter: getenv("SPEICHE_HEARTBEAT_BACKOFF_AFTER_FAILURES", "3"),
	}
}

func getenv(key string, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}

	return value
}
