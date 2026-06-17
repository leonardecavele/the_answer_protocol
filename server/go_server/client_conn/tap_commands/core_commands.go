package tap_commands

import (
	"errors"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
)

func handleConnectCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
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
		return protocol.ResponseInvalidUsername, nil
	}

	if response := client.Room.SetUsername(client, strings.ToUpper(args)); response != "" {
		return response, nil
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "CONNECT",
		Arguments: "",
	}); err != nil && !errors.Is(err, serverError.ErrGameServerNotConnected) {
		return "", err
	}

	client.Room.BroadcastEvent(protocol.EventBatch{
		IgnoredPlayers: []string{client.Username},
		Events: []protocol.Event{
			{
				EmittedBy: client.Username,
				EventName: "CONNECT",
			},
		},
	})

	return protocol.ResponseConnected, nil
}

func handleLookCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, false); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "LOOK",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := protocol.HandleCommandError("LOOK", response.ErrorCode); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleMoveCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, true, true); response != "" || err != nil {
		return response, err
	}

	if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "MOVE",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := protocol.HandleCommandError("MOVE", response.ErrorCode); errorResponse != "" {
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
				Command:   "MOVE",
				Arguments: args,
			}); err != nil {
				return "", err
			}
		}
	}

	return "OK " + response.Data, nil
}

func handleQuitCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, false); response != "" || err != nil {
		return response, err
	}

	return protocol.ResponseBye, nil
}
