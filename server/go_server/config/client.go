package config

import "time"

const (
	MaxConnection           = RoomSize
	MaxConnectionAttempts   = 5
	ConnectionAttemptWindow = 10 * time.Second
	MaxCommandsPerWindow    = 20
	CommandRateWindow       = time.Second
	MaxFloodPoints          = 5
	FloodPointDecayInterval = time.Hour
	AuthenticationTimeout   = 30 * time.Second
	ClientReadTimeout       = 30 * time.Minute
)
