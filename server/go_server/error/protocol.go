package error

import (
	"fmt"
	"go_server/protocol"
)

func HandleGameCommandError(errorCode int) string {
	switch errorCode {
	case 0:
		return ""
	case 201:
		return protocol.ResponseUsernameAlreadyUsed
	case 301:
		return protocol.ResponseNoExit
	case 400:
		return protocol.ResponseInvalidArguments
	case 401:
		return protocol.ResponseNotInGroup
	case 402:
		return protocol.ResponseAlreadyInGroup
	case 405:
		return protocol.ResponseNpcNotHostile
	case 406:
		return protocol.ResponseNoQuestAvailable
	case 900:
		return protocol.ResponseConnectionFailed
	case 901:
		return protocol.ResponseSendFailed
	default:
		return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", errorCode)
	}
}
