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

func groupCreate(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, false); response != "" || err != nil {
		return response, err
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

func groupInvite(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, true); response != "" || err != nil {
		return response, err
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
		EmittedBy: client.Username,
		EventName: "GROUP INVITE",
	})

	return "OK", nil
}

func groupJoin(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, true); response != "" || err != nil {
		return response, err
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
	client.Group.BroadcastEvent(protocol.EventBatch{
		IgnoredPlayers: []string{client.Username},
		Events: []protocol.Event{
			{
				EmittedBy: client.Username,
				EventName: "GROUP JOIN",
			},
		},
	})
	return "OK group=" + client.Group.Id, nil
}

func groupLeave(args string, client *session.Client, gameServer *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServer, false, false); response != "" || err != nil {
		return response, err
	}
	if client.Group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	groupedClients := client.Group.GroupedClients()
	client.QuitGroup()
	for _, groupedClient := range groupedClients {
		if groupedClient == client {
			continue
		}
		client.Room.RouteEvent(groupedClient.Username, protocol.Event{
			EmittedBy: client.Username,
			EventName: "GROUP LEAVE",
		})
	}
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
