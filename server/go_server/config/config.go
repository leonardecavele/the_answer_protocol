package config

var QuitCommands = map[string]struct{}{
	"quit": {},
	"exit": {},
	"q":    {},
}

const (
	LogFormat = "15:04:05.000000"

	GameServerIP = "localhost"

	GoServerPort   = 38800
	GameServerPort = 38801

	ProtocolVersion = 1

	RoomSize = 20

	GameConnectionRetryDelay = 5

	GameConfirmationMessage = "Duly noted."
)
