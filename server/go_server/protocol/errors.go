package protocol

import (
	"fmt"
	serverError "go_server/error"
	"strings"
)

func ResponseError(errorCode int, message string) string {
	return fmt.Sprintf("ERR %03d %s", errorCode, message)
}

// Connection
var (
	ResponseGameServerClosed = ResponseError(serverError.GameServerClosedError, "GAME_SERVER_UNAVAILABLE")
)

// Client/session validation
var (
	ResponseNotConnected     = ResponseError(serverError.NotConnectedError, "NOT_CONNECTED")
	ResponseAlreadyConnected = ResponseError(serverError.AlreadyConnectedError, "ALREADY_CONNECTED")
	ResponseInvalidUsername  = ResponseError(serverError.InvalidUsernameError, "INVALID_USERNAME")
	ResponseRoomFull         = ResponseError(serverError.RoomFullError, "ROOM_FULL")
	ResponseGroupFull        = ResponseError(serverError.GroupFullError, "GROUP_FULL")
)

// Command validation
var (
	ResponseEmptyCommand     = ResponseError(serverError.EmptyCommandError, "EMPTY_COMMAND")
	ResponseCommandNotFound  = ResponseError(serverError.CommandNotFoundError, "COMMAND_NOT_FOUND")
	ResponseInvalidArguments = ResponseError(serverError.InvalidArgumentsError, "INVALID_ARGUMENTS")
	ResponseInvalidScope     = ResponseError(serverError.InvalidScopeError, "INVALID_SCOPE")
	ResponseInvalidQuestion  = ResponseError(serverError.InvalidQuestionError, "INVALID_QUESTION")
	ResponseInvalidCommand   = ResponseError(serverError.InvalidCommandError, "INVALID_COMMAND")
)

// RFC 42TAP standard errors
var (
	ResponseUsernameAlreadyUsed = ResponseError(serverError.NameInUseError, "NAME_IN_USE")
	ResponseNoExit              = ResponseError(serverError.NoExitError, "NO_EXIT")
	ResponseNotInGroup          = ResponseError(serverError.NotInGroupError, "NOT_IN_GROUP")
	ResponseAlreadyInGroup      = ResponseError(serverError.AlreadyInGroupError, "ALREADY_IN_GROUP")
	ResponseItemNotFound        = ResponseError(serverError.ItemNotFoundError, "ITEM_NOT_FOUND")
	ResponseItemNotInInventory  = ResponseError(serverError.ItemNotInInventoryError, "ITEM_NOT_IN_INVENTORY")
	ResponseNpcNotFound         = ResponseError(serverError.NpcNotFoundError, "NPC_NOT_FOUND")
	ResponseNpcNotHostile       = ResponseError(serverError.NpcNotHostileError, "NPC_NOT_HOSTILE")
	ResponseNoQuestAvailable    = ResponseError(serverError.NoQuestAvailableError, "NO_QUEST_AVAILABLE")
	ResponseConnectionFailed    = ResponseError(serverError.ConnectionFailedError, "CONNECTION_FAILED")
	ResponseSendFailed          = ResponseError(serverError.SendFailedError, "SEND_FAILED")
)

// TAP documented extension errors
var (
	ResponseNoSuchUser     = ResponseError(serverError.NoSuchUserError, "NO_SUCH_USER")
	ResponseNotInvited     = ResponseError(serverError.NotInvitedError, "NOT_INVITED")
	ResponseNotGroupLeader = ResponseError(serverError.NotGroupLeaderError, "NOT_GROUP_LEADER")
	ResponseGroupNotFound  = ResponseError(serverError.GroupNotFoundError, "GROUP_NOT_FOUND")
	ResponseNoSuchGroup    = ResponseError(serverError.NoSuchGroupError, "NO_SUCH_GROUP")
)

var ErrorResponseByCommand = map[string]map[int]string{
	"CONNECT": {
		serverError.ProtocolNoError:       "",
		serverError.NameInUseError:        ResponseUsernameAlreadyUsed,
		serverError.AlreadyConnectedError: ResponseAlreadyConnected,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"LOOK": {
		serverError.ProtocolNoError:       "",
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"MOVE": {
		serverError.ProtocolNoError:       "",
		serverError.NoExitError:           ResponseNoExit,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUIT": {
		serverError.ProtocolNoError:       "",
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"CHAT": {
		serverError.ProtocolNoError:       "",
		serverError.InvalidScopeError:     ResponseInvalidScope,
		serverError.NotInGroupError:       ResponseNotInGroup,
		serverError.NoSuchUserError:       ResponseNoSuchUser,
		serverError.NoSuchGroupError:      ResponseNoSuchGroup,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP CREATE": {
		serverError.ProtocolNoError:       "",
		serverError.AlreadyInGroupError:   ResponseAlreadyInGroup,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP INVITE": {
		serverError.ProtocolNoError:       "",
		serverError.NotInGroupError:       ResponseNotInGroup,
		serverError.NoSuchUserError:       ResponseNoSuchUser,
		serverError.AlreadyInGroupError:   ResponseAlreadyInGroup,
		serverError.GroupNotFoundError:    ResponseGroupNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP JOIN": {
		serverError.ProtocolNoError:       "",
		serverError.AlreadyInGroupError:   ResponseAlreadyInGroup,
		serverError.NotInvitedError:       ResponseNotInvited,
		serverError.GroupNotFoundError:    ResponseGroupNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP LEAVE": {
		serverError.ProtocolNoError:       "",
		serverError.NotInGroupError:       ResponseNotInGroup,
		serverError.GroupNotFoundError:    ResponseGroupNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"TAKE": {
		serverError.ProtocolNoError:       "",
		serverError.ItemNotFoundError:     ResponseItemNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"DROP": {
		serverError.ProtocolNoError:         "",
		serverError.ItemNotInInventoryError: ResponseItemNotInInventory,
		serverError.ConnectionFailedError:   ResponseConnectionFailed,
		serverError.SendFailedError:         ResponseSendFailed,
		serverError.InvalidQuestionError:    ResponseInvalidQuestion,
		serverError.InvalidCommandError:     ResponseInvalidCommand,
	},
	"INVENTORY": {
		serverError.ProtocolNoError:       "",
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"TALK": {
		serverError.ProtocolNoError:       "",
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"ATTACK": {
		serverError.ProtocolNoError:       "",
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.NpcNotHostileError:    ResponseNpcNotHostile,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"STATUS": {
		serverError.ProtocolNoError:       "",
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUEST": {
		serverError.ProtocolNoError:       "",
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.NoQuestAvailableError: ResponseNoQuestAvailable,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUESTS": {
		serverError.ProtocolNoError:       "",
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
}

func HandleGameCommandError(command string, errorCode int) string {
	if responsesByCode, ok := ErrorResponseByCommand[strings.ToUpper(command)]; ok {
		if response, ok := responsesByCode[errorCode]; ok {
			return response
		}
	}

	return ResponseError(errorCode, "UNKNOWN_ERROR")
}
