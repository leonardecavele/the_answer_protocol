package protocol

import (
	"fmt"
	serverError "go_server/error"
	"strings"
)

func ResponseError(errorCode int, message string) string {
	if errorCode <= serverError.NoError || errorCode > 999 {
		errorCode = serverError.UnknownError
		message = "UNKNOWN_ERROR"
	}

	return fmt.Sprintf("ERR %03d %s", errorCode, message)
}

var (
	ResponseNotConnected        = ResponseError(serverError.NotConnectedError, "NOT_CONNECTED")
	ResponseAlreadyConnected    = ResponseError(serverError.AlreadyConnectedError, "ALREADY_CONNECTED")
	ResponseInvalidUsername     = ResponseError(serverError.InvalidUsernameError, "INVALID_USERNAME")
	ResponseRoomFull            = ResponseError(serverError.RoomFullError, "ROOM_FULL")
	ResponseGroupFull           = ResponseError(serverError.GroupFullError, "GROUP_FULL")
	ResponseEmptyCommand        = ResponseError(serverError.EmptyCommandError, "EMPTY_COMMAND")
	ResponseCommandNotFound     = ResponseError(serverError.CommandNotFoundError, "COMMAND_NOT_FOUND")
	ResponseInvalidArguments    = ResponseError(serverError.InvalidArgumentsError, "INVALID_ARGUMENTS")
	ResponseInvalidScope        = ResponseError(serverError.InvalidScopeError, "INVALID_SCOPE")
	ResponseInvalidQuestion     = ResponseError(serverError.InvalidQuestionError, "INVALID_QUESTION")
	ResponseInvalidCommand      = ResponseError(serverError.InvalidCommandError, "INVALID_COMMAND")
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
	ResponseGameServerClosed    = ResponseConnectionFailed
	ResponseSendFailed          = ResponseError(serverError.SendFailedError, "SEND_FAILED")
	ResponseGameServerTimeout   = ResponseError(serverError.GameServerTimeoutError, "GAME_SERVER_TIMEOUT")
	ResponseNoSuchUser          = ResponseError(serverError.NoSuchUserError, "NO_SUCH_USER")
	ResponseNotInvited          = ResponseError(serverError.NotInvitedError, "NOT_INVITED")
	ResponseNotGroupLeader      = ResponseError(serverError.NotGroupLeaderError, "NOT_GROUP_LEADER")
	ResponseGroupNotFound       = ResponseError(serverError.GroupNotFoundError, "GROUP_NOT_FOUND")
	ResponseNoSuchGroup         = ResponseError(serverError.NoSuchGroupError, "NO_SUCH_GROUP")
	ResponseUnknownError        = ResponseError(serverError.UnknownError, "UNKNOWN_ERROR")
)

var ErrorResponseByCommand = map[string]map[int]string{
	"CONNECT": {
		serverError.NameInUseError:        ResponseUsernameAlreadyUsed,
		serverError.AlreadyConnectedError: ResponseAlreadyConnected,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"LOOK": {
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"MOVE": {
		serverError.NoExitError:           ResponseNoExit,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUIT": {
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"CHAT": {
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
		serverError.AlreadyInGroupError:   ResponseAlreadyInGroup,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP INVITE": {
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
		serverError.AlreadyInGroupError:   ResponseAlreadyInGroup,
		serverError.NotInvitedError:       ResponseNotInvited,
		serverError.GroupNotFoundError:    ResponseGroupNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"GROUP LEAVE": {
		serverError.NotInGroupError:       ResponseNotInGroup,
		serverError.GroupNotFoundError:    ResponseGroupNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"TAKE": {
		serverError.ItemNotFoundError:     ResponseItemNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"DROP": {
		serverError.ItemNotInInventoryError: ResponseItemNotInInventory,
		serverError.ConnectionFailedError:   ResponseConnectionFailed,
		serverError.SendFailedError:         ResponseSendFailed,
		serverError.InvalidQuestionError:    ResponseInvalidQuestion,
		serverError.InvalidCommandError:     ResponseInvalidCommand,
	},
	"INVENTORY": {
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"TALK": {
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"ATTACK": {
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.NpcNotHostileError:    ResponseNpcNotHostile,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"STATUS": {
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUEST": {
		serverError.NpcNotFoundError:      ResponseNpcNotFound,
		serverError.NoQuestAvailableError: ResponseNoQuestAvailable,
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
	"QUESTS": {
		serverError.ConnectionFailedError: ResponseConnectionFailed,
		serverError.SendFailedError:       ResponseSendFailed,
		serverError.InvalidQuestionError:  ResponseInvalidQuestion,
		serverError.InvalidCommandError:   ResponseInvalidCommand,
	},
}

func HandleCommandError(command string, errorCode int) string {
	if errorCode == serverError.NoError {
		return ""
	}

	if errorCode == serverError.GameServerTimeoutError {
		return ResponseGameServerTimeout
	}

	if responsesByCode, ok := ErrorResponseByCommand[strings.ToUpper(command)]; ok {
		if response, ok := responsesByCode[errorCode]; ok {
			return response
		}
	}

	return ResponseUnknownError
}
