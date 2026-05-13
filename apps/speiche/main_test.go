package main

import "testing"

func TestNextHeartbeatIntervalBacksOffAfterThreshold(t *testing.T) {
	if got := nextHeartbeatInterval(30, 300, 3, 2); got != 30 {
		t.Fatalf("before threshold: got %d", got)
	}
	if got := nextHeartbeatInterval(30, 300, 3, 3); got != 60 {
		t.Fatalf("at threshold: got %d", got)
	}
	if got := nextHeartbeatInterval(240, 300, 3, 6); got != 300 {
		t.Fatalf("must cap at max: got %d", got)
	}
}
