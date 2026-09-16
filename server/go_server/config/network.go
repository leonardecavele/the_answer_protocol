package config

import "time"

const (
	GoServerPort      = 38800
	TCPWriteTimeout   = 5 * time.Second
	ReadStringMaxSize = 65536
)
