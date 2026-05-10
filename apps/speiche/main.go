package main

import (
	"fmt"
	"log"

	"codeberg.org/KyleHub/nabe/apps/speiche/internal/config"
	"codeberg.org/KyleHub/nabe/apps/speiche/internal/drivers/adguard"
	"codeberg.org/KyleHub/nabe/apps/speiche/internal/heartbeat"
)

func main() {
	cfg := config.FromEnv()
	client := heartbeat.NewClient(cfg.NabeAPIURL, cfg.EnrollmentToken)
	driver := adguard.NewDriver(cfg.AdGuardURL)

	log.Printf("speiche starting: node=%s api=%s public_server=false", cfg.NodeName, cfg.NabeAPIURL)
	fmt.Println(client.Status())
	fmt.Println(driver.Status())
}
