package client_conn

import (
	"go_server/config"
	"go_server/game_conn"
	"go_server/logger"
	"strings"
	"sync"
)

type Room struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

func NewRoom() *Room {
	return &Room{
		clients: make(map[string]*Client, config.RoomSize),
	}
}

func (room *Room) SetUsername(client *Client, username string) string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	if client.State == AUTHENTICATED {
		return responseAlreadyConnected
	}

	if len(room.clients) >= config.RoomSize {
		return responseRoomFull
	}

	if _, ok := room.clients[username]; ok {
		return responseUsernameAlreadyUsed
	}
	client.Username = username
	client.State = AUTHENTICATED
	room.clients[username] = client

	return ""
}

func (room *Room) EraseUsername(client *Client) {
	room.mutex.Lock()
	if client.State == AUTHENTICATED {
		delete(room.clients, client.Username)
	}
	room.mutex.Unlock()
}

func (room *Room) ConnectedUsernames() []string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	usernames := make([]string, 0, len(room.clients))
	for username := range room.clients {
		usernames = append(usernames, username)
	}

	return usernames
}

func (room *Room) RouteCommand(username string, command string) bool {
	room.mutex.Lock()
	client, ok := room.clients[strings.ToUpper(username)]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.commandChan <- command
	return true
}

func (room *Room) RouteEvent(username string, event string) bool {
	room.mutex.Lock()
	client, ok := room.clients[strings.ToUpper(username)]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.eventChan <- event
	return true
}

func (room *Room) ReconnectPlayersToGameServer(gameServer *game_conn.GameServerManager) error {
	for _, username := range room.ConnectedUsernames() {
		command := game_conn.CommandToGameServer{
			Player:    username,
			Command:   "CONNECT",
			Arguments: username,
		}

		if err := gameServer.WriteCommand(command); err != nil {
			return err
		}
		logger.AppLogger.Info("Reconnected %s to Game server", username)
	}

	return nil
}
