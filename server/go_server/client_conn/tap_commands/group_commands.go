package tap_commands

import (
	"errors"
	serverError "go_server/error"
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

func groupCreate(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, false); response != "" || err != nil {
		return response, err
	}
	if client.GetGroup() != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}

	group, err := session.NewGroup(client)
	if err != nil {
		return "", err
	}

	return "OK group=" + group.Id, nil
}

func groupInvite(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}
	group := client.GetGroup()
	if group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	invitedClient, ok := client.Room.GetClient(args)
	if !ok {
		return protocol.ResponseNoSuchUser, nil
	}
	if invitedClient.GetGroup() != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}

	if gameServerManager.IsConnected() {
		inSameRoom, err := client.InSameRoom([]*session.Client{invitedClient}, gameServerManager)
		if err != nil {
			return "", err
		}
		if !inSameRoom {
			return protocol.ResponseNotInSameRoom, nil
		}

		if response := group.Invite(invitedClient.Username); response != "" {
			return response, nil
		}
	}

	client.Room.RouteEvent(invitedClient.Username, protocol.Event{
		EmittedBy: client.Username,
		EventName: "GROUP INVITE",
	})

	return "OK", nil
}

func groupJoin(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, true, true); response != "" || err != nil {
		return response, err
	}
	if client.GetGroup() != nil {
		return protocol.ResponseAlreadyInGroup, nil
	}

	groupLeader, ok := client.Room.GetClient(args)
	if !ok {
		return protocol.ResponseNoSuchUser, nil
	}
	group := groupLeader.GetGroup()
	if group == nil || !groupLeader.IsLeader() {
		return protocol.ResponseGroupNotFound, nil
	}

	if gameServerManager.IsConnected() {
		inSameRoom, err := client.InSameRoom([]*session.Client{groupLeader}, gameServerManager)
		if err != nil {
			return "", err
		}
		if !inSameRoom {
			return protocol.ResponseNotInSameRoom, nil
		}

		if response := client.JoinGroup(group); response != "" {
			return response, nil
		}
	}

	group.BroadcastEvent(protocol.EventBatch{
		IgnoredPlayers: []string{client.Username},
		Events: []protocol.Event{
			{
				EmittedBy: client.Username,
				EventName: "GROUP JOIN",
			},
		},
	})

	return "OK group=" + group.Id, nil
}

func groupLeave(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	if response, err := isOk(args, client, gameServerManager, false, false); response != "" || err != nil {
		return response, err
	}
	group := client.GetGroup()
	if group == nil {
		return protocol.ResponseNotInGroup, nil
	}

	groupedClients := group.GroupedClients()
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

	if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "GROUP LEAVE",
		Arguments: "",
	}); err != nil && !errors.Is(err, serverError.ErrGameServerNotConnected) {
		return "", err
	}

	return "OK", nil
}

func handleGroupCommand(args string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	subCommand, subArgs, hasSubArguments := strings.Cut(args, " ")
	if subCommand == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	subCommandHandler, ok := groupCommands[strings.ToUpper(subCommand)]
	if !ok {
		return protocol.ResponseCommandNotFound, nil
	}
	if hasSubArguments && subArgs == "" {
		return protocol.ResponseInvalidArguments, nil
	}

	return subCommandHandler(subArgs, client, gameServerManager)
}
