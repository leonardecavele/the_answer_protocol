package client_conn

import (
	"go_server/config"
	"strconv"
)

// Success
var (
	// Connection.
	responseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion)
	responseConnected = "OK connected"
	responseBye       = "OK bye"
)

// Error
var (
	// Connection.
	responseRustServerShutdown = "ERR 900 RUST_SERVER_SHUTDOWN"

	// Client/session validation.
	responseNotConnected     = "ERR 400 NOT_CONNECTED"
	responseAlreadyConnected = "ERR 400 ALREADY_CONNECTED"
	responseInvalidUsername  = "ERR 400 INVALID_USERNAME"
	responseRoomFull         = "ERR 400 ROOM_FULL"

	// Command validation.
	responseEmptyCommand     = "ERR 400 EMPTY_COMMAND"
	responseCommandNotFound  = "ERR 400 COMMAND_NOT_FOUND"
	responseInvalidArguments = "ERR 400 INVALID_ARGUMENTS"
	responseInvalidScope     = "ERR 400 INVALID_SCOPE"

	// RFC 42TAP standard errors.
	responseUsernameAlreadyUsed = "ERR 201 NAME_IN_USE"
	responseNoExit              = "ERR 301 NO_EXIT"
	responseNotInGroup          = "ERR 401 NOT_IN_GROUP"
	responseAlreadyInGroup      = "ERR 402 ALREADY_IN_GROUP"
	responseItemNotFound        = "ERR 404 ITEM_NOT_FOUND"
	responseItemNotInInventory  = "ERR 404 ITEM_NOT_IN_INVENTORY"
	responseNpcNotFound         = "ERR 404 NPC_NOT_FOUND"
	responseNpcNotHostile       = "ERR 405 NPC_NOT_HOSTILE"
	responseNoQuestAvailable    = "ERR 406 NO_QUEST_AVAILABLE"
	responseConnectionFailed    = "ERR 900 CONNECTION_FAILED"
	responseSendFailed          = "ERR 901 SEND_FAILED"

	// TAP documented extension errors.
	responseNoSuchUser    = "ERR 403 NO_SUCH_USER"
	responseNotInvited    = "ERR 403 NOT_INVITED"
	responseGroupNotFound = "ERR 404 GROUP_NOT_FOUND"
	responseNoSuchGroup   = "ERR 404 NO_SUCH_GROUP"
)
