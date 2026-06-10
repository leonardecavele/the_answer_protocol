package client_conn

import (
	"go_server/config"
	"strconv"
)

// Success
var (
	// Connection
	responseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion)
	responseConnected = "OK connected"
	responseBye       = "OK bye"
)

// Error
var (
	//Connection
	responseNotConnected = "ERR 211 NOT CONNECTED"

	// Command
	responseEmptyCommand     = "ERR 2929 empty command"
	responseCommandNotFound  = "ERR 404 COMMAND_NOT_FOUND"
	responseInvalidArguments = "ERR 200 PLACEHOLDER INVALID ARGUMENTS"

	// Client username
	responseInvalidUsername     = "ERR 6060 INVALID USERNAME PLACEHOLDER"
	responseAlreadyConnected    = "ERR 1313 PLACEHOLDER ALREADY CONNECTED"
	responseUsernameAlreadyUsed = "ERR 201 NAME IN USED"

	// Room
	responseRoomFull = "ERR 1292 PLACEHOLDER ROOM FULL"
)
