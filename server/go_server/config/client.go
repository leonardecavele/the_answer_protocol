package config

import "time"

const (
	MaxConnection           = RoomSize + 1
	MaxCommandsPerWindow    = 25
	CommandRateWindow       = time.Second
	MaxConnectionAttempts   = 20
	ConnectionAttemptWindow = CommandRateWindow
	MaxFloodPoints          = 5
	FloodPointDecayInterval = 30 * time.Minute
	AuthenticationTimeout   = 30 * time.Second
	ClientReadTimeout       = 10 * time.Minute
)
