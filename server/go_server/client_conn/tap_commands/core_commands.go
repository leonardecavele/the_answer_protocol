package tap_commands

import (
	"errors"
	"go_server/client_conn"
	"go_server/game_conn"
	"strings"
)

func handleConnectCommand(args string, client *client_conn.Client, gameServer *game_conn.GameServerManager) (string, error) {
	isValidUsername := func(username string) bool {
		if username == "" {
			return false
		}
		for _, c := range username {
			if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
				(c >= '0' && c <= '9') || c == '-' || c == '_' {
				continue
			}
			return false
		}
		return true
	}

	if !isValidUsername(args) {
		return client_conn.ResponseInvalidUsername, nil
	}

	if response := client.Room.SetUsername(client, strings.ToUpper(args)); response != "" {
		return response, nil
	}

	command := game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "CONNECT",
		Arguments: args,
	}

	if err := gameServer.WriteCommand(command); err != nil && !errors.Is(err, game_conn.ErrGameServerNotConnected) {
		return "", err
	}

	return client_conn.ResponseConnected, nil
}

func handleLookCommand(args string, client *client_conn.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return client_conn.ResponseGameServerClosed, nil
	}

	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}
	if args != "" {
		return client_conn.ResponseInvalidArguments, nil
	}

	command := game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "LOOK",
		Arguments: args,
	}

	if err := gameServer.WriteCommand(command); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := handleGameCommandError(response); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleMoveCommand(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuitCommand(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if args != "" {
		return client_conn.ResponseInvalidArguments, nil
	}

	return client_conn.ResponseBye, nil
}
