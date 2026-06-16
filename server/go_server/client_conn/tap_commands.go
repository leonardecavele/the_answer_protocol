package client_conn

import (
	"errors"
	"fmt"
	"go_server/game_conn"
	"strings"
)

type handleTapCommandArgs func(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error)

var tapCommands = map[string]handleTapCommandArgs{
	// CORE
	"CONNECT": handleConnectCommand,
	"LOOK":    handleLookCommand,
	"MOVE":    handleMoveCommand,
	"QUIT":    handleQuitCommand,

	// COMMUNICATION
	"CHAT": handleChatCommand,
	"WHO":  handleWhoCommand,

	// GROUP
	"GROUP": handleGroupCommand,

	// RESOURCE INTERACTION
	"TAKE":      handleTakeCommand,
	"DROP":      handleDropCommand,
	"INVENTORY": handleInventoryCommand,
	"TALK":      handleTalkCommand,
	"ATTACK":    handleAttackCommand,
	"STATUS":    handleStatusCommand,
	"QUEST":     handleQuestCommand,
	"QUESTS":    handleQuestsCommand,
}

func handleGameCommandError(response game_conn.CommandFromGameServer) string {
	switch response.ErrorCode {
	case 201:
		return responseUsernameAlreadyUsed
	case 301:
		return responseNoExit
	case 400:
		return responseInvalidArguments
	case 401:
		return responseNotInGroup
	case 402:
		return responseAlreadyInGroup
	case 405:
		return responseNpcNotHostile
	case 406:
		return responseNoQuestAvailable
	case 900:
		return responseConnectionFailed
	case 901:
		return responseSendFailed
	default:
		return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", response.ErrorCode)
	}
}

// CORE

func handleConnectCommand(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
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
		return responseInvalidUsername, nil
	}

	if response := client.room.SetUsername(client, strings.ToUpper(args)); response != "" {
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

	return responseConnected, nil
}

func handleLookCommand(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return responseGameServerClosed, nil
	}

	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
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

func handleMoveCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuitCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if args != "" {
		return responseInvalidArguments, nil
	}

	return responseBye, nil
}

// COMMUNICATION

type handleChatScope func(client *Client, message string) string

var chatScopes = map[string]handleChatScope{
	"GLOBAL":  chatGlobalScope,
	"ROOM":    chatRoomScope,
	"GROUP":   chatGroupScope,
	"PRIVATE": chatPrivateScope,
}

func chatGroupScope(client *Client, message string) string {
	if client.group == nil {
		return responseNotInGroup
	}

	client.group.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GROUP CHAT",
		Data:      client.Username + " " + message,
	})
	return "OK"
}

func chatGlobalScope(client *Client, message string) string {
	client.room.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GLOBAL CHAT",
		Data:      client.Username + " " + message,
	})

	return "OK"
}

func chatRoomScope(client *Client, message string) string {
	//client.group.BroadcastEvent(game_conn.EventFromGameServer{
	//	Player:    client.Username,
	//	EventName: "GROUP CHAT",
	//	Data:      client.Username + " " + message,
	//})

	return "NOT IMPLEMENTED YET"
}

func chatPrivateScope(client *Client, message string) string {
	username, privateMessage, ok := strings.Cut(message, " ")
	if !ok || username == "" || strings.TrimSpace(privateMessage) == "" {
		return responseInvalidArguments
	}

	ok = client.room.RouteEvent(username, game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "PRIVATE CHAT",
		Data:      client.Username + " " + privateMessage,
	})
	if !ok {
		return responseNoSuchUser
	}

	return "OK"
}

func handleChatCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}

	scope, message, ok := strings.Cut(args, " ")
	if !ok || scope == "" {
		return responseInvalidArguments, nil
	}

	chatScopeHandler, ok := chatScopes[strings.ToUpper(scope)]
	if !ok {
		return responseInvalidScope, nil
	}

	if strings.TrimSpace(message) == "" {
		return responseInvalidArguments, nil
	}

	return chatScopeHandler(client, message), nil
}

func handleWhoCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

// GROUP

type handleGroup = handleTapCommandArgs

var groupCommands = map[string]handleGroup{
	"CREATE": groupCreate,
	"INVITE": groupInvite,
	"JOIN":   groupJoin,
	"LEAVE":  groupLeave,
	"QUIT":   groupLeave,
}

func groupCreate(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
	}
	if client.group != nil {
		return responseAlreadyInGroup, nil
	}

	group, err := NewGroup(client.Username)
	if err != nil {
		return "", err
	}

	if response := client.JoinGroup(group); response != "" {
		return response, nil
	}
	return "OK group=" + group.id, nil
}

func groupInvite(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return responseInvalidArguments, nil
	}
	if client.group == nil {
		return responseNotInGroup, nil
	}

	invitedClient, ok := client.room.GetClient(args)
	if !ok {
		return responseNoSuchUser, nil
	}
	if invitedClient.group != nil {
		return responseAlreadyInGroup, nil
	}

	client.room.RouteEvent(invitedClient.Username, game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GROUP INVITE",
		Data:      client.Username,
	})

	return "OK", nil
}

func groupJoin(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return responseInvalidArguments, nil
	}
	if client.group != nil {
		return responseAlreadyInGroup, nil
	}

	leader, ok := client.room.GetClient(args)
	if !ok {
		return responseNoSuchUser, nil
	}
	if leader.group == nil {
		return responseGroupNotFound, nil
	}

	if response := client.JoinGroup(leader.group); response != "" {
		return response, nil
	}
	return "OK group=" + client.group.id, nil
}

func groupLeave(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
	}
	if client.group == nil {
		return responseNotInGroup, nil
	}

	client.QuitGroup()
	return "OK", nil
}

func handleGroupCommand(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
	subCommand, subArgs, _ := strings.Cut(args, " ")
	if subCommand == "" {
		return responseInvalidArguments, nil
	}

	subCommandHandler, ok := groupCommands[strings.ToUpper(subCommand)]
	if !ok {
		return responseCommandNotFound, nil
	}

	return subCommandHandler(subArgs, client, gameServer)
}

// RESOURCE INTERACTION

func handleTakeCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleDropCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleInventoryCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleTalkCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleAttackCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleStatusCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestsCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
