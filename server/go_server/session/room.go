package session

import (
	"go_server/config"
	"go_server/game_conn"
	"go_server/logger"
	"go_server/protocol"
	"sort"
	"strings"
	"sync"
)

type Room struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

func usernameKey(username string) string {
	return strings.ToUpper(strings.TrimSpace(username))
}

func NewRoom() *Room {
	return &Room{
		clients: make(map[string]*Client, config.RoomSize),
	}
}

func (room *Room) SetUsername(client *Client, username string) string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	if client.IsAuthenticated() {
		return protocol.ResponseAlreadyConnected
	}

	if len(room.clients) >= config.RoomSize {
		return protocol.ResponseRoomFull
	}

	key := usernameKey(username)
	if _, ok := room.clients[key]; ok {
		return protocol.ResponseUsernameAlreadyUsed
	}
	client.authenticate(username)
	room.clients[key] = client

	return ""
}

func (room *Room) RollbackUsername(client *Client) {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	username, state, _ := client.connectionInfo()
	key := usernameKey(username)
	if state != AUTHENTICATED || room.clients[key] != client {
		return
	}

	delete(room.clients, key)
	client.rollbackAuthentication()
}

func (room *Room) UnregisterClient(client *Client) {
	room.mutex.Lock()
	if client.IsAuthenticated() {
		delete(room.clients, usernameKey(client.Username))
	}
	room.mutex.Unlock()
}

func (room *Room) ConnectedUsernames() []string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	usernames := make([]string, 0, len(room.clients))
	for _, client := range room.clients {
		usernames = append(usernames, client.Username)
	}

	return usernames
}

func (room *Room) Count() int {
	room.mutex.Lock()
	defer room.mutex.Unlock()
	return len(room.clients)
}

func (room *Room) GetClient(username string) (*Client, bool) {
	room.mutex.Lock()
	client, ok := room.clients[usernameKey(username)]
	room.mutex.Unlock()

	return client, ok
}

func (room *Room) GetGroup(id string) (*Group, bool) {
	room.mutex.Lock()
	clients := make([]*Client, 0, len(room.clients))
	for _, client := range room.clients {
		clients = append(clients, client)
	}
	room.mutex.Unlock()

	for _, client := range clients {
		group := client.GetGroup()
		if group != nil && strings.EqualFold(group.Id, id) {
			return group, true
		}
	}
	return nil, false
}

func (room *Room) Disconnect(username string) bool {
	client, ok := room.GetClient(username)
	if !ok {
		return false
	}

	_ = client.Disconnect()
	return true
}

func (room *Room) Groups() []GroupInfo {
	if room == nil {
		return nil
	}

	room.mutex.Lock()
	clients := make([]*Client, 0, len(room.clients))
	for _, client := range room.clients {
		clients = append(clients, client)
	}
	room.mutex.Unlock()

	groupsByPointer := make(map[*Group]struct{})
	for _, client := range clients {
		if group := client.GetGroup(); group != nil {
			groupsByPointer[group] = struct{}{}
		}
	}

	groups := make([]GroupInfo, 0, len(groupsByPointer))
	for group := range groupsByPointer {
		if info, ok := group.Info(); ok {
			groups = append(groups, info)
		}
	}
	sort.Slice(groups, func(i, j int) bool {
		return groups[i].ID < groups[j].ID
	})
	return groups
}

func (room *Room) RouteCommand(username string, command game_conn.CommandFromGameServer) bool {
	room.mutex.Lock()
	client, ok := room.clients[usernameKey(username)]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.SendCommand(command)
	return true
}

func (room *Room) RouteEvent(username string, event protocol.Event) bool {
	username = usernameKey(username)

	room.mutex.Lock()
	client, ok := room.clients[username]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.SendEvent(event)
	return true
}

func (room *Room) BroadcastEvent(eventBatch protocol.EventBatch) {
	ignored := make(map[string]struct{}, len(eventBatch.IgnoredPlayers))
	for _, username := range eventBatch.IgnoredPlayers {
		username = usernameKey(username)
		if username != "" {
			ignored[username] = struct{}{}
		}
	}

	target := usernameKey(eventBatch.Player)
	room.mutex.Lock()
	clients := make([]*Client, 0, len(room.clients))
	if target != "" {
		if _, ok := ignored[target]; !ok {
			if client, ok := room.clients[target]; ok {
				clients = append(clients, client)
			}
		}
	} else {
		for username, client := range room.clients {
			if _, ok := ignored[username]; ok {
				continue
			}
			clients = append(clients, client)
		}
	}
	room.mutex.Unlock()

	for _, client := range clients {
		for _, event := range eventBatch.Events {
			client.SendEvent(event)
		}
	}
}

func (room *Room) ReconnectPlayersToGameServer(gameServerManager *game_conn.GameServerManager) error {
	for _, username := range room.ConnectedUsernames() {
		if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
			Player:    username,
			Command:   "CONNECT",
			Arguments: username,
		}); err != nil {
			return err
		}
		logger.AppLogger.Info("Reconnected %s to Game server", username)
	}

	room.BroadcastEvent(protocol.EventBatch{
		Events: []protocol.Event{
			{
				EventName: "GAME SERVER",
				Data:      "CONNECTED",
			},
		},
	})

	return nil
}
