package tap_commands

import (
	"fmt"
	"go_server/client_conn"
	"go_server/game_conn"
)

type handleTapCommandArgs func(args string, client *client_conn.Client, gameServer *game_conn.GameServerManager) (string, error)

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
		return client_conn.ResponseUsernameAlreadyUsed
	case 301:
		return client_conn.ResponseNoExit
	case 400:
		return client_conn.ResponseInvalidArguments
	case 401:
		return client_conn.ResponseNotInGroup
	case 402:
		return client_conn.ResponseAlreadyInGroup
	case 405:
		return client_conn.ResponseNpcNotHostile
	case 406:
		return client_conn.ResponseNoQuestAvailable
	case 900:
		return client_conn.ResponseConnectionFailed
	case 901:
		return client_conn.ResponseSendFailed
	default:
		return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", response.ErrorCode)
	}
}
