package main

import (
	"fmt"
	"os"

	"github.com/KyleHub-Dev/nabe/apps/cli/internal/commands"
	"github.com/KyleHub-Dev/nabe/apps/cli/internal/config"
)

func main() {
	cfg := config.FromEnv()
	if err := commands.Run(os.Args[1:], cfg); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
