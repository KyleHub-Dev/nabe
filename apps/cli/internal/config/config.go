package config

import "os"

type Config struct {
	APIURL string
}

func FromEnv() Config {
	apiURL := os.Getenv("NABE_API_URL")
	if apiURL == "" {
		apiURL = "http://localhost:8080"
	}

	return Config{APIURL: apiURL}
}
