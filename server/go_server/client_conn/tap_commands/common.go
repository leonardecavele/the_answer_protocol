package tap_commands

import (
	"fmt"
	"go_server/config"
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

var customTapCommands = map[string]handleTapCommandArgs{
	"USE":   handleUseCommand,
	"AGGRO": handleAggroCommand,
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
	customTapCommands,
)

func isOk(args string, client *session.Client, gameServer *game_conn.GameServerManager, needGameServer bool, hasArgs bool) (string, error) {
	if needGameServer && !gameServer.IsConnected() {
		return protocol.ResponseGameServerClosed, nil
	}

	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	if hasArgs {
		if args == "" || strings.TrimSpace(args) != args {
			return protocol.ResponseInvalidArguments, nil
		}
	} else if args != "" {
		return protocol.ResponseInvalidArguments, nil
	}

	return "", nil
}

func sendGroupedOrSolo(command string, args string, client *session.Client, gameServer *game_conn.GameServerManager) error {
	if client.Group == nil {
		return gameServer.WriteCommand(game_conn.CommandToGameServer{
			Player:    client.Username,
			Command:   command,
			Arguments: args,
		})
	}

	groupedClients := client.Group.GroupedClients()
	groupedPlayers := make([]string, 0, len(groupedClients))
	for _, groupedClient := range groupedClients {
		if groupedClient == client {
			continue
		}
		groupedPlayers = append(groupedPlayers, groupedClient.Username)
	}

	return gameServer.WriteCommand(game_conn.GroupedCommandToGameServer{
		Leader:         client.Username,
		GroupedPlayers: groupedPlayers,
		Command:        command,
		Arguments:      args,
	})
}

func readGameServerCommand(command string, client *session.Client) (game_conn.CommandFromGameServer, string) {
	response, ok := client.ReadCommandTimeout(config.GameServerCommandTimeout)
	if !ok {
		return game_conn.CommandFromGameServer{}, protocol.ResponseGameServerTimeout
	}

	if errorResponse := protocol.HandleCommandError(command, response.ErrorCode); errorResponse != "" {
		return game_conn.CommandFromGameServer{}, errorResponse
	}

	return response, ""
}
