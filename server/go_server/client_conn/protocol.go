package client_conn

import (
	"go_server/config"
	"strconv"
)

// Success
var (
	// Connection.
	ResponseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion)
	ResponseConnected = "OK connected"
	ResponseBye       = "OK bye"
)

// Error
var (
	// Connection.
	ResponseGameServerClosed = "ERR 900 GAME_SERVER_CLOSE"

	// Client/session validation.
	ResponseNotConnected     = "ERR 400 NOT_CONNECTED"
	ResponseAlreadyConnected = "ERR 400 ALREADY_CONNECTED"
	ResponseInvalidUsername  = "ERR 400 INVALID_USERNAME"
	ResponseRoomFull         = "ERR 400 ROOM_FULL"

	// Command validation.
	ResponseEmptyCommand     = "ERR 400 EMPTY_COMMAND"
	ResponseCommandNotFound  = "ERR 400 COMMAND_NOT_FOUND"
	ResponseInvalidArguments = "ERR 400 INVALID_ARGUMENTS"
	ResponseInvalidScope     = "ERR 400 INVALID_SCOPE"

	// RFC 42TAP standard errors.
	ResponseUsernameAlreadyUsed = "ERR 201 NAME_IN_USE"
	ResponseNoExit              = "ERR 301 NO_EXIT"
	ResponseNotInGroup          = "ERR 401 NOT_IN_GROUP"
	ResponseAlreadyInGroup      = "ERR 402 ALREADY_IN_GROUP"
	ResponseItemNotFound        = "ERR 404 ITEM_NOT_FOUND"
	ResponseItemNotInInventory  = "ERR 404 ITEM_NOT_IN_INVENTORY"
	ResponseNpcNotFound         = "ERR 404 NPC_NOT_FOUND"
	ResponseNpcNotHostile       = "ERR 405 NPC_NOT_HOSTILE"
	ResponseNoQuestAvailable    = "ERR 406 NO_QUEST_AVAILABLE"
	ResponseConnectionFailed    = "ERR 900 CONNECTION_FAILED"
	ResponseSendFailed          = "ERR 901 SEND_FAILED"

	// TAP documented extension errors.
	ResponseNoSuchUser    = "ERR 403 NO_SUCH_USER"
	ResponseNotInvited    = "ERR 403 NOT_INVITED"
	ResponseGroupNotFound = "ERR 404 GROUP_NOT_FOUND"
	ResponseNoSuchGroup   = "ERR 404 NO_SUCH_GROUP"
)

//events
var (
	// Room.
	EventRoomPresenceEnter = "EVT ROOM PRESENCE ENTER <username>"
	EventRoomPresenceLeave = "EVT ROOM PRESENCE LEAVE <username>"
	EventRoomChat          = "EVT ROOM CHAT <username> <message>"

	// Global.
	EventGlobalChat = "EVT GLOBAL CHAT <username> <message>"

	// Group.
	EventGroupInvite = "EVT GROUP INVITE <leader>"
	EventGroupJoin   = "EVT GROUP JOIN <username>"
	EventGroupLeave  = "EVT GROUP LEAVE <username>"
	EventGroupChat   = "EVT GROUP CHAT <username> <message>"

	// Stats.
	EventStatsPlayers = "EVT STATS players=<count>"
)
