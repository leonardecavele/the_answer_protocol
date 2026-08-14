package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
)

type handleFight = handleTapCommandArgs

var fightCommands = map[string]handleFight{
	"ATTACK": fightAttack,
	"CREATE": fightCreate,
}

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

func fightAttack(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "FIGHT_ATTACK",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("FIGHT_ATTACK", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func fightCreate(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}

	if client.Group != nil && !client.IsLeader() {
		return protocol.ResponseNotGroupLeader, nil
	}

	if err := sendGroupedOrSolo("FIGHT_CREATE", args, client, gameServerManager); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("FIGHT_CREATE", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleFightCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	subCommand, subArgs, _ := strings.Cut(args, " ")
	if subCommand == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	subCommandHandler, ok := fightCommands[strings.ToUpper(subCommand)]
	if !ok {
		return protocol.ResponseCommandNotFound, nil
	}

	return subCommandHandler(subArgs, client, gameServerManager)
}
