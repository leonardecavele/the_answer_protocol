package config

import "time"

const (
	MaxConnection           = RoomSize
	MaxCommandsPerWindow    = 20
	CommandRateWindow       = time.Second
	MaxConnectionAttempts   = MaxCommandsPerWindow
	ConnectionAttemptWindow = CommandRateWindow
	MaxFloodPoints          = 5
	FloodPointDecayInterval = time.Hour
	AuthenticationTimeout   = 30 * time.Second
	ClientReadTimeout       = 30 * time.Minute
)
