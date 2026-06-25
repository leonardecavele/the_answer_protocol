package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
)

func handleTakeCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "TAKE",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("TAKE", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + "taken=" + response.Data, nil
}

func handleDropCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "DROP",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("DROP", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + "dropped=" + response.Data, nil
}

func handleInventoryCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, false); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:  client.Username,
		Command: "INVENTORY",
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("INVENTORY", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleTalkCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "TALK",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("TALK", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleAttackCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "ATTACK",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("ATTACK", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleStatusCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, false); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:  client.Username,
		Command: "STATUS",
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("STATUS", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleQuestCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}
	if client.Group != nil && !client.IsLeader() {
		return protocol.ResponseNotGroupLeader, nil
	}

	if err := sendGroupedOrSolo("QUEST", args, client, gameServer); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("QUEST", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleQuestsCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, false); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:  client.Username,
		Command: "QUESTS",
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("QUESTS", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}
