package tap_commands

import (
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
)

func handleTakeCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return protocol.ResponseGameServerClosed, nil
	}

	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	if strings.Contains(args, " ") || args == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "TAKE",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := serverError.HandleGameCommandError(response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + "taken=" + response.Data, nil
}

func handleDropCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return protocol.ResponseGameServerClosed, nil
	}

	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	if strings.Contains(args, " ") || args == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "DROP",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := serverError.HandleGameCommandError(response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + "dropped=" + response.Data, nil
}

func handleInventoryCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return protocol.ResponseGameServerClosed, nil
	}

	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	if args != "" {
		return protocol.ResponseInvalidArguments, nil
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:  client.Username,
		Command: "INVENTORY",
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := serverError.HandleGameCommandError(response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleTalkCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleAttackCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleStatusCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	// grouped
	return "", nil
}

func handleQuestsCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
