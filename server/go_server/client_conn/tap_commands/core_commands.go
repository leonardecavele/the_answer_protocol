package tap_commands

import (
	"errors"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
)

func handleConnectCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	isValidUsername := func(username string) bool {
		if username == "" {
			return false
		}
		if len(username) < 3 || len(username) > 20 {
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

	if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
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

	if !gameServerManager.IsConnected() {
		client.SendEvent(protocol.Event{
			EventName: "GAME SERVER",
			Data:      "DISCONNECTED",
		})
	}

	return protocol.ResponseConnected, nil
}

func handleLookCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, false); response != "" || err != nil {
		return response, err
	}

	if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "LOOK",
		Arguments: args,
	}); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("LOOK", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleMoveCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}

	if client.Group != nil && !client.IsLeader() {
		return protocol.ResponseNotGroupLeader, nil
	}

	if err := sendGroupedOrSolo("MOVE", args, client, gameServerManager); err != nil {
		return "", err
	}

	response, errorResponse := readGameServerCommand("MOVE", client)
	if errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + "room=" + response.Data, nil
}

func handleQuitCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, false, false); response != "" || err != nil {
		return response, err
	}

	return protocol.ResponseBye, nil
}
