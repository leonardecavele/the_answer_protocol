package config

import "time"

const (
	GameServerIP   = "10.11.12.2"
	GameServerPort = 38801

	GameConnectionRetryDelay = 5
	GameServerCommandTimeout = 5 * time.Second

	GameConfirmationMessage = "Duly noted."
)
