package client_conn

import (
	"go_server/config"
	"strconv"
)

// Success
var (
	// Connection
	responseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion) + "\n"
	responseConnected = "OK connected\n"
)

// Error
var (
	// Command
	responseCommandNotFound = "ERR 400 COMMAND_NOT_FOUND\n"

	// Client username
	responseInvalidUsername     = "ERR 6060 INVALID USERNAME PLACEHOLDER\n"
	responseAlreadyConnected    = "ERR 1313 PLACEHOLDER ALREADY CONNECTED\n"
	responseUsernameAlreadyUsed = "ERR 9090 PLACEHOLDER NAME TAKEN\n"

	// Room
	responseRoomFull = "ERR 1292 PLACEHOLDER ROOM FULL\n"
)
