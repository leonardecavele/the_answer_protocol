package config

import "time"

const (
	MaxConnection           = RoomSize
	MaxConnectionAttempts   = 5
	ConnectionAttemptWindow = 10 * time.Second
	MaxCommandsPerWindow    = 10
	CommandRateWindow       = time.Second
	AuthenticationTimeout   = 30 * time.Second
	ClientReadTimeout       = 30 * time.Minute
)
