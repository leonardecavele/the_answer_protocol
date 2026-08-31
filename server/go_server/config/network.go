package config

import "time"

const (
	GoServerPort      = 38800
	TCPWriteTimeout   = 5 * time.Second
	ReadStringMaxSize = 4096
)
