package config

import "time"

const (
	MaxConnection         = RoomSize
	AuthenticationTimeout = 30 * time.Second
	ClientReadTimeout     = 30 * time.Minute
)
