package tap_commands

import (
	"go_server/client_conn"
	"go_server/game_conn"
	"strings"
)

type handleGroup = handleTapCommandArgs

var groupCommands = map[string]handleGroup{
	"CREATE": groupCreate,
	"INVITE": groupInvite,
	"JOIN":   groupJoin,
	"LEAVE":  groupLeave,
	"QUIT":   groupLeave,
}

func groupCreate(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}
	if args != "" {
		return client_conn.ResponseInvalidArguments, nil
	}
	if client.Group != nil {
		return client_conn.ResponseAlreadyInGroup, nil
	}

	group, err := client_conn.NewGroup(client.Username)
	if err != nil {
		return "", err
	}

	if response := client.JoinGroup(group); response != "" {
		return response, nil
	}
	return "OK group=" + group.Id, nil
}

func groupInvite(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return client_conn.ResponseInvalidArguments, nil
	}
	if client.Group == nil {
		return client_conn.ResponseNotInGroup, nil
	}

	invitedClient, ok := client.Room.GetClient(args)
	if !ok {
		return client_conn.ResponseNoSuchUser, nil
	}
	if invitedClient.Group != nil {
		return client_conn.ResponseAlreadyInGroup, nil
	}

	client.Room.RouteEvent(invitedClient.Username, game_conn.EventFromGameServer{
		Player:    client.Username,
		EventName: "GROUP INVITE",
		Data:      client.Username,
	})

	return "OK", nil
}

func groupJoin(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return client_conn.ResponseInvalidArguments, nil
	}
	if client.Group != nil {
		return client_conn.ResponseAlreadyInGroup, nil
	}

	leader, ok := client.Room.GetClient(args)
	if !ok {
		return client_conn.ResponseNoSuchUser, nil
	}
	if leader.Group == nil {
		return client_conn.ResponseGroupNotFound, nil
	}

	if response := client.JoinGroup(leader.Group); response != "" {
		return response, nil
	}
	return "OK group=" + client.Group.Id, nil
}

func groupLeave(args string, client *client_conn.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != client_conn.AUTHENTICATED {
		return client_conn.ResponseNotConnected, nil
	}
	if args != "" {
		return client_conn.ResponseInvalidArguments, nil
	}
	if client.Group == nil {
		return client_conn.ResponseNotInGroup, nil
	}

	client.QuitGroup()
	return "OK", nil
}

func handleGroupCommand(args string, client *client_conn.Client, gameServer *game_conn.GameServerManager) (string, error) {
	subCommand, subArgs, _ := strings.Cut(args, " ")
	if subCommand == "" {
		return client_conn.ResponseInvalidArguments, nil
	}

	subCommandHandler, ok := groupCommands[strings.ToUpper(subCommand)]
	if !ok {
		return client_conn.ResponseCommandNotFound, nil
	}

	return subCommandHandler(subArgs, client, gameServer)
}
