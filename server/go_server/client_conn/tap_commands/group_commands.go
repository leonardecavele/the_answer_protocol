package tap_commands

import (
	"go_server/game_conn"
	"go_server/protocol"
	"go_server/session"
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

func groupCreate(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}
	if args != "" {
		return protocol.ResponseInvalidArguments, nil
	}
	if client.Group != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}

	group, err := session.NewGroup(client)
	if err != nil {
		return "", err
	}

	return "OK group=" + group.Id, nil
}

func groupInvite(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return protocol.ResponseInvalidArguments, nil
	}
	if client.Group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	invitedClient, ok := client.Room.GetClient(args)
	if !ok {
		return protocol.ResponseNoSuchUser, nil
	}
	if invitedClient.Group != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}
	if response := client.Group.Invite(invitedClient.Username); response != "" {
		return response, nil
	}

	client.Room.RouteEvent(invitedClient.Username, protocol.Event{
		Players:        []string{invitedClient.Username},
		IgnoredPlayers: []string{},
		EmittedBy:      client.Username,
		EventName:      "GROUP INVITE",
		Data:           client.Username,
	})

	return "OK", nil
}

func groupJoin(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}
	if args == "" || strings.Contains(args, " ") {
		return protocol.ResponseInvalidArguments, nil
	}
	if client.Group != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}

	groupMember, ok := client.Room.GetClient(args)
	if !ok {
		return protocol.ResponseNoSuchUser, nil
	}
	if groupMember.Group == nil {
		return protocol.ResponseGroupNotFound, nil
	}

	if response := client.JoinGroup(groupMember.Group); response != "" {
		return response, nil
	}
	return "OK group=" + client.Group.Id, nil
}

func groupLeave(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	if client.State != session.AUTHENTICATED {
		return protocol.ResponseNotConnected, nil
	}
	if args != "" {
		return protocol.ResponseInvalidArguments, nil
	}
	if client.Group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	client.QuitGroup()
	return "OK", nil
}

func handleGroupCommand(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	subCommand, subArgs, _ := strings.Cut(args, " ")
	if subCommand == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	subCommandHandler, ok := groupCommands[strings.ToUpper(subCommand)]
	if !ok {
		return protocol.ResponseCommandNotFound, nil
	}

	return subCommandHandler(subArgs, client, gameServer)
}
