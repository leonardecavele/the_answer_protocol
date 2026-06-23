package tap_commands

import (
	"encoding/json"
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

	groupedClients := client.Group.GroupedClients()

	players := make([]string, 0, len(groupedClients))
	for _, groupedClient := range groupedClients {
		players = append(players, groupedClient.Username)
	}

	fight := game_conn.FightInstance{
		MobId:   args,
		Players: players,
	}

	data, err := json.Marshal(fight)
	if err != nil {
		return "", err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "AGGRO",
		Arguments: string(data),
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("AGGRO", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}
