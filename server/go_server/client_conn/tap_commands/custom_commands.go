package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
)

func handleUseCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "USE",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("USE", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleAggroCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}

	if client.Group != nil && !client.IsLeader() {
		return protocol.ResponseNotGroupLeader, nil
	}

	if err := sendGroupedOrSolo("AGGRO", args, client, gameServerManager); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("AGGRO", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}
