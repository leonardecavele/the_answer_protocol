package config

var QuitCommands = map[string]struct{}{
	"quit": {},
	"exit": {},
	"q":    {},
}

const (
	LogFormat = "2006-01-02 15:04:05.000"

	RustServerIP = "127.0.0.1"

	GoServerPort   = 38800
	RustServerPort = 38801

	ProtocolVersion = 12

	RoomSize = 2
)
