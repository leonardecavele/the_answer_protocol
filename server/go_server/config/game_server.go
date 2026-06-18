package config

import "time"

const (
	GameServerIP   = "localhost"
	GameServerPort = 38801

	GameConnectionRetryDelay = 5
	GameServerCommandTimeout = 3 * time.Second

	GameConfirmationMessage = "Duly noted."
)
