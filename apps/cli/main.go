package main

import (
	"fmt"
	"os"

	"codeberg.org/KyleHub/nabe/apps/cli/internal/commands"
	"codeberg.org/KyleHub/nabe/apps/cli/internal/config"
)

func main() {
	cfg := config.FromEnv()
	if err := commands.Run(os.Args[1:], cfg); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
