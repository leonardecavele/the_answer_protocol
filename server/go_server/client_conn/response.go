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
	//Connection
	responseNotConnected = "ERR 211 NOT CONNECTED\n"

	// Command
	responseCommandNotFound  = "ERR 404 COMMAND_NOT_FOUND\n"
	responseInvalidArguments = "ERR 200 PLACEHOLDER INVALID ARGUMENTS\n"

	// Client username
	responseInvalidUsername     = "ERR 6060 INVALID USERNAME PLACEHOLDER\n"
	responseAlreadyConnected    = "ERR 1313 PLACEHOLDER ALREADY CONNECTED\n"
	responseUsernameAlreadyUsed = "ERR 201 NAME IN USED\n"

	// Room
	responseRoomFull = "ERR 1292 PLACEHOLDER ROOM FULL\n"
)
