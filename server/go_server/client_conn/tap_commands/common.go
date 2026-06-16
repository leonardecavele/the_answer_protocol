package tap_commands

import (
	"fmt"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
)

type handleTapCommandArgs func(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error)

var TapCommands = map[string]handleTapCommandArgs{
	// CORE
	"CONNECT": handleConnectCommand,
	"LOOK":    handleLookCommand,
	"MOVE":    handleMoveCommand,
	"QUIT":    handleQuitCommand,

	// COMMUNICATION
	"CHAT": handleChatCommand,
	"WHO":  handleWhoCommand,

	// GROUP
	"GROUP": handleGroupCommand,

	// RESOURCE INTERACTION
	"TAKE":      handleTakeCommand,
	"DROP":      handleDropCommand,
	"INVENTORY": handleInventoryCommand,
	"TALK":      handleTalkCommand,
	"ATTACK":    handleAttackCommand,
	"STATUS":    handleStatusCommand,
	"QUEST":     handleQuestCommand,
	"QUESTS":    handleQuestsCommand,
}

func handleGameCommandError(response game_conn.CommandFromGameServer) string {
	switch response.ErrorCode {
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
		return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", response.ErrorCode)
	}
}
