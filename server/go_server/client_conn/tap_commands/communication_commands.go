package tap_commands

import (
	"go_server/client_conn"
	"go_server/game_conn"
	"strings"
)

type handleChatScope func(client *client_conn.Client, message string) string

var chatScopes = map[string]handleChatScope{
	"GLOBAL":  chatGlobalScope,
	"ROOM":    chatRoomScope,
	"GROUP":   chatGroupScope,
	"PRIVATE": chatPrivateScope,
}

func chatGroupScope(client *client_conn.Client, message string) string {
	if client.Group == nil {
		return client_conn.ResponseNotInGroup
	}

	client.Group.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GROUP CHAT",
		Data:      client.Username + " " + message,
	})
	return "OK"
}

func chatGlobalScope(client *client_conn.Client, message string) string {
	client.Room.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GLOBAL CHAT",
		Data:      client.Username + " " + message,
	})

	return "OK"
}

func chatRoomScope(client *client_conn.Client, message string) string {
	//client.group.BroadcastEvent(game_conn.EventFromGameServer{
	//	Player:    client.Username,
	//	EventName: "GROUP CHAT",
	//	Data:      client.Username + " " + message,
	//})

	return "NOT IMPLEMENTED YET"
}

func chatPrivateScope(client *client_conn.Client, message string) string {
	username, privateMessage, ok := strings.Cut(message, " ")
	if !ok || username == "" || strings.TrimSpace(privateMessage) == "" {
		return client_conn.ResponseInvalidArguments
	}

	ok = client.Room.RouteEvent(username, game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "PRIVATE CHAT",
		Data:      client.Username + " " + privateMessage,
	})
	if !ok {
		return client_conn.ResponseNoSuchUser
	}

	return "OK"
}

func handleChatCommand(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}

	scope, message, ok := strings.Cut(args, " ")
	if !ok || scope == "" {
		return client_conn.ResponseInvalidArguments, nil
	}

	chatScopeHandler, ok := chatScopes[strings.ToUpper(scope)]
	if !ok {
		return client_conn.ResponseInvalidScope, nil
	}

	if strings.TrimSpace(message) == "" {
		return client_conn.ResponseInvalidArguments, nil
	}

	return chatScopeHandler(client, message), nil
}

func handleWhoCommand(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
