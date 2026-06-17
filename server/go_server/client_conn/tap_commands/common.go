package tap_commands

import (
	"fmt"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
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

func isOk(args string, client *session.Client, gameServer *game_conn.GameServerManager, needGameServer bool, hasArgs bool) (string, error) {
	if needGameServer && !gameServer.IsConnected() {
		return protocol.ResponseGameServerClosed, nil
	}

	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	if (hasArgs && (strings.Contains(args, " ") || args == "")) || (!hasArgs && args != "") {
		return protocol.ResponseInvalidArguments, nil
	}

	return "", nil
}
