package config

import "time"

const (
	GameServerIP   = "10.12.5.5"
	GameServerPort = 38801

	GameConnectionRetryDelay = 5
	GameServerCommandTimeout = 3 * time.Second

	GameConfirmationMessage = "Duly noted."
)
