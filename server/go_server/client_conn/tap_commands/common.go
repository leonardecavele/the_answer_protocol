package tap_commands

import (
	"fmt"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
)

type handleTapCommandArgs func(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error)

var coreTapCommands = map[string]handleTapCommandArgs{
	"CONNECT": handleConnectCommand,
	"LOOK":    handleLookCommand,
	"MOVE":    handleMoveCommand,
	"QUIT":    handleQuitCommand,
}

var communicationTapCommands = map[string]handleTapCommandArgs{
	"CHAT": handleChatCommand,
	"WHO":  handleWhoCommand,
}

var groupTapCommands = map[string]handleTapCommandArgs{
	"GROUP": handleGroupCommand,
}

var resourceInteractionTapCommands = map[string]handleTapCommandArgs{
	"TAKE":      handleTakeCommand,
	"DROP":      handleDropCommand,
	"INVENTORY": handleInventoryCommand,
	"TALK":      handleTalkCommand,
	"ATTACK":    handleAttackCommand,
	"STATUS":    handleStatusCommand,
	"QUEST":     handleQuestCommand,
	"QUESTS":    handleQuestsCommand,
}

var TapCommands = func(commandGroups ...map[string]handleTapCommandArgs) map[string]handleTapCommandArgs {
	commands := make(map[string]handleTapCommandArgs)
	for _, commandGroup := range commandGroups {
		for command, handler := range commandGroup {
			if _, ok := commands[command]; ok {
				panic(fmt.Sprintf("duplicate tap command %q", command))
			}
			commands[command] = handler
		}
	}
	return commands
}(
	coreTapCommands,
	communicationTapCommands,
	groupTapCommands,
	resourceInteractionTapCommands,
)

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
