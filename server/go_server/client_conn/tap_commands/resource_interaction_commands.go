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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("TAKE", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("DROP", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("INVENTORY", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("TALK", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("ATTACK", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("STATUS", response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleQuestCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "QUEST",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("QUEST", response.ErrorCode); errorResponse != "" {
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

	response := client.ReadCommand()
	if errorResponse := protocol.HandleGameCommandError("QUESTS", response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}
