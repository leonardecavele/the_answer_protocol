package error

import (
	"fmt"
	"go_server/protocol"
	"strings"
)

const (
	ProtocolNoError = 0

	ProtocolNameInUse = 201
	ProtocolNoExit    = 301

	ProtocolAlreadyConnected = 400
	ProtocolInvalidScope     = 400

	ProtocolNotInGroup     = 401
	ProtocolAlreadyInGroup = 402

	ProtocolNoSuchUser = 403
	ProtocolNotInvited = 403

	ProtocolItemNotFound       = 404
	ProtocolItemNotInInventory = 404
	ProtocolNpcNotFound        = 404
	ProtocolGroupNotFound      = 404
	ProtocolNoSuchGroup        = 404

	ProtocolNpcNotHostile    = 405
	ProtocolNoQuestAvailable = 406

	ProtocolConnectionFailed = 900
	ProtocolSendFailed       = 901

	ProtocolInvalidQuestion = 998
	ProtocolInvalidCommand  = 999
)

var protocolErrorsByCommand = map[string]map[int]string{
	"CONNECT": {
		ProtocolNoError:          "",
		ProtocolNameInUse:        protocol.ResponseUsernameAlreadyUsed,
		ProtocolAlreadyConnected: protocol.ResponseAlreadyConnected,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"LOOK": {
		ProtocolNoError:          "",
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"MOVE": {
		ProtocolNoError:          "",
		ProtocolNoExit:           protocol.ResponseNoExit,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"QUIT": {
		ProtocolNoError:          "",
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"CHAT": {
		ProtocolNoError:          "",
		ProtocolInvalidScope:     protocol.ResponseInvalidScope,
		ProtocolNotInGroup:       protocol.ResponseNotInGroup,
		ProtocolNoSuchUser:       protocol.ResponseNoSuchUser,
		ProtocolNoSuchGroup:      protocol.ResponseNoSuchGroup,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"GROUP CREATE": {
		ProtocolNoError:          "",
		ProtocolAlreadyInGroup:   protocol.ResponseAlreadyInGroup,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"GROUP INVITE": {
		ProtocolNoError:          "",
		ProtocolNotInGroup:       protocol.ResponseNotInGroup,
		ProtocolNoSuchUser:       protocol.ResponseNoSuchUser,
		ProtocolAlreadyInGroup:   protocol.ResponseAlreadyInGroup,
		ProtocolGroupNotFound:    protocol.ResponseGroupNotFound,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"GROUP JOIN": {
		ProtocolNoError:          "",
		ProtocolAlreadyInGroup:   protocol.ResponseAlreadyInGroup,
		ProtocolNotInvited:       protocol.ResponseNotInvited,
		ProtocolGroupNotFound:    protocol.ResponseGroupNotFound,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"GROUP LEAVE": {
		ProtocolNoError:          "",
		ProtocolNotInGroup:       protocol.ResponseNotInGroup,
		ProtocolGroupNotFound:    protocol.ResponseGroupNotFound,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"TAKE": {
		ProtocolNoError:          "",
		ProtocolItemNotFound:     protocol.ResponseItemNotFound,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"DROP": {
		ProtocolNoError:            "",
		ProtocolItemNotInInventory: protocol.ResponseItemNotInInventory,
		ProtocolConnectionFailed:   protocol.ResponseConnectionFailed,
		ProtocolSendFailed:         protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:    protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:     protocol.ResponseInvalidCommand,
	},
	"INVENTORY": {
		ProtocolNoError:          "",
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"TALK": {
		ProtocolNoError:          "",
		ProtocolNpcNotFound:      protocol.ResponseNpcNotFound,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"ATTACK": {
		ProtocolNoError:          "",
		ProtocolNpcNotFound:      protocol.ResponseNpcNotFound,
		ProtocolNpcNotHostile:    protocol.ResponseNpcNotHostile,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"STATUS": {
		ProtocolNoError:          "",
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"QUEST": {
		ProtocolNoError:          "",
		ProtocolNpcNotFound:      protocol.ResponseNpcNotFound,
		ProtocolNoQuestAvailable: protocol.ResponseNoQuestAvailable,
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
	"QUESTS": {
		ProtocolNoError:          "",
		ProtocolConnectionFailed: protocol.ResponseConnectionFailed,
		ProtocolSendFailed:       protocol.ResponseSendFailed,
		ProtocolInvalidQuestion:  protocol.ResponseInvalidQuestion,
		ProtocolInvalidCommand:   protocol.ResponseInvalidCommand,
	},
}

func HandleGameCommandError(command string, errorCode int) string {
	if responsesByCode, ok := protocolErrorsByCommand[strings.ToUpper(command)]; ok {
		if response, ok := responsesByCode[errorCode]; ok {
			return response
		}
	}

	return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", errorCode)
}
