package commands

import (
	"errors"
	"fmt"

	"codeberg.org/KyleHub/nabe/apps/cli/internal/config"
)

func Run(args []string, cfg config.Config) error {
	if len(args) == 0 {
		return usage()
	}

	switch args[0] {
	case "version":
		fmt.Println("nabe cli 0.0.0")
		return nil
	case "status":
		fmt.Printf("nabe api: %s\n", cfg.APIURL)
		return nil
	default:
		return usage()
	}
}

func usage() error {
	return errors.New("usage: nabe <version|status>")
}
