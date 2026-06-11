package config

var QuitCommands = map[string]struct{}{
	"quit": {},
	"exit": {},
	"q":    {},
}

const (
	LogFormat = "15:04:05.000000"

	RustServerIP = "10.14.9.6"

	GoServerPort   = 38800
	RustServerPort = 38801

	ProtocolVersion = 1

	RoomSize = 20

	RustConnectionRetryDelay = 1

	RustConfirmationMessage = "Duly noted."
)
