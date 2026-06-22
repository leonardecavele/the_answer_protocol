package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
)

func handleUseCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
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

func handleAggroCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if client.Group != nil && !client.IsLeader() {
		return protocol.ResponseNotGroupLeader, nil
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "AGGRO",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("AGGRO", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	if client.Group != nil {
		clients := client.Group.GroupedClients()
		for _, c := range clients {
			if c == client {
				continue
			}
			if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
				Player:    c.Username,
				Command:   "AGGRO",
				Arguments: args,
			}); err != nil {
				return "", err
			}
		}
	}

	return "OK " + response.Data, nil
}
