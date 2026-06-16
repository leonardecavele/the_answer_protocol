package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
	"strings"
)

type handleChatScope func(client *session.Client, message string) string

var chatScopes = map[string]handleChatScope{
	"GLOBAL":  chatGlobalScope,
	"ROOM":    chatRoomScope,
	"GROUP":   chatGroupScope,
	"PRIVATE": chatPrivateScope,
}

func chatGroupScope(client *session.Client, message string) string {
	if client.Group == nil {
		return protocol.ResponseNotInGroup
	}

	client.Group.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GROUP CHAT",
		Data:      client.Username + " " + message,
	})
	return "OK"
}

func chatGlobalScope(client *session.Client, message string) string {
	client.Room.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GLOBAL CHAT",
		Data:      client.Username + " " + message,
	})

	return "OK"
}

func chatRoomScope(client *session.Client, message string) string {
	//client.group.BroadcastEvent(game_conn.EventFromGameServer{
	//	Player:    client.Username,
	//	EventName: "GROUP CHAT",
	//	Data:      client.Username + " " + message,
	//})

	return "NOT IMPLEMENTED YET"
}

func chatPrivateScope(client *session.Client, message string) string {
	username, privateMessage, ok := strings.Cut(message, " ")
	if !ok || username == "" || strings.TrimSpace(privateMessage) == "" {
		return protocol.ResponseInvalidArguments
	}

	ok = client.Room.RouteEvent(username, game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "PRIVATE CHAT",
		Data:      client.Username + " " + privateMessage,
	})
	if !ok {
		return protocol.ResponseNoSuchUser
	}

	return "OK"
}

func handleChatCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
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

	return chatScopeHandler(client, message), nil
}

func handleWhoCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
