package config

import "time"

const (
	MaxConnection           = RoomSize
	MaxCommandsPerWindow    = 25
	CommandRateWindow       = time.Second
	MaxConnectionAttempts   = 20
	ConnectionAttemptWindow = CommandRateWindow
	MaxFloodPoints          = 5
	FloodPointDecayInterval = 30 * time.Minute
	AuthenticationTimeout   = 30 * time.Second
	ClientReadTimeout       = 30 * time.Minute
)
