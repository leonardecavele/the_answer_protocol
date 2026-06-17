package tap_commands

import (
	"encoding/json"
	"go_server/game_conn"
	"go_server/helper"
	"go_server/protocol"
	"go_server/session"
	"strconv"
	"strings"
)

type chatScope func(client *session.Client, message string, gameServer *game_conn.GameServerManager) (string, error)

var chatScopes = map[string]chatScope{
	"GLOBAL":  chatGlobalScope,
	"ROOM":    chatRoomScope,
	"GROUP":   chatGroupScope,
	"PRIVATE": chatPrivateScope,
}

func chatGroupScope(client *session.Client, message string, _ *game_conn.GameServerManager) (string, error) {
	if client.Group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	client.Group.BroadcastEvent(protocol.Event{
		IgnoredPlayers: []string{client.Username},
		EmittedBy:      client.Username,
		EventName:      "GROUP CHAT",
		Data:           message,
	})
	return "OK", nil
}

func chatGlobalScope(client *session.Client, message string, _ *game_conn.GameServerManager) (string, error) {
	client.Room.BroadcastEvent(protocol.Event{
		IgnoredPlayers: []string{client.Username},
		EmittedBy:      client.Username,
		EventName:      "GLOBAL CHAT",
		Data:           message,
	})

	return "OK", nil
}

func chatRoomScope(client *session.Client, message string, gameServer *game_conn.GameServerManager) (string, error) {
	id, err := helper.NewID()
	if err != nil {
		return "", err
	}

	answer, err := gameServer.AskQuestion(game_conn.QuestionToGameServer{
		Question: "ROOM_PLAYERS",
		Data:     client.Username,
		Id:       id,
	})
	if err != nil {
		return "", err
	}

	var usernames []string
	if err := json.Unmarshal([]byte(answer.Data), &usernames); err != nil {
		return "", err
	}

	routed := make(map[string]struct{}, len(usernames))
	for _, username := range usernames {
		username = strings.ToUpper(strings.TrimSpace(username))
		if username == "" || username == client.Username {
			continue
		}
		if _, ok := routed[username]; ok {
			continue
		}
		routed[username] = struct{}{}

		client.Room.RouteEvent(username, protocol.Event{
			Players:        []string{username},
			IgnoredPlayers: []string{},
			EmittedBy:      client.Username,
			EventName:      "ROOM CHAT",
			Data:           message,
		})
	}

	return "OK", nil
}

func chatPrivateScope(client *session.Client, message string, _ *game_conn.GameServerManager) (string, error) {
	username, privateMessage, ok := strings.Cut(message, " ")
	if !ok || username == "" || strings.TrimSpace(privateMessage) == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	ok = client.Room.RouteEvent(username, protocol.Event{
		Players:        []string{strings.ToUpper(username)},
		IgnoredPlayers: []string{},
		EmittedBy:      client.Username,
		EventName:      "PRIVATE CHAT",
		Data:           privateMessage,
	})
	if !ok {
		return protocol.ResponseNoSuchUser, nil
	}

	return "OK", nil
}

func handleChatCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}

	scope, message, ok := strings.Cut(args, " ")
	if !ok || scope == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	chatScopeHandler, ok := chatScopes[strings.ToUpper(scope)]
	if !ok {
		return protocol.ResponseInvalidScope, nil
	}

	if strings.TrimSpace(message) == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	return chatScopeHandler(client, message, gameServer)
}

func handleWhoCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, false); response != "" || err != nil {
		return response, err
	}

	return "OK " + "players=" + strconv.Itoa(client.Room.Count()), nil
}
