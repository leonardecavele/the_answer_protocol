package config

var QuitCommands = map[string]struct{}{
	"quit": {},
	"exit": {},
	"q":    {},
}

const (
	LogFormat = "15:04:05"

	RustServerIP = "127.0.0.1"

	GoServerPort   = 38800
	RustServerPort = 38801

	ProtocolVersion = 1

	RoomSize = 2
)
