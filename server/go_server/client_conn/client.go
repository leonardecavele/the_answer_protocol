package client_conn

import (
	"errors"
	"go_server/rust_conn"
	"net"
	"strings"
	"sync"
)

import (
	"go_server/config"
)

type ClientState string

const (
	CONNECTED     ClientState = "CONNECTED"
	AUTHENTICATED ClientState = "AUTHENTICATED"
)

type Client struct {
	Conn        net.Conn
	Id          string
	Username    string
	State       ClientState
	commandChan chan string
	eventChan   chan string
	writeMutex  sync.Mutex
}

func NewClient(conn net.Conn) *Client {
	return &Client{
		Conn:        conn,
		Id:          conn.RemoteAddr().String(),
		State:       CONNECTED,
		commandChan: make(chan string, 16),
		eventChan:   make(chan string, 16),
	}
}

func (c *Client) EraseClient(rustServer *rust_conn.RustServerManager) error {
	username := c.Username
	state := c.State

	c.EraseUsername()
	closeErr := c.Conn.Close()

	if state == AUTHENTICATED {
		command := rust_conn.CommandToRust{
			Player:    username,
			Command:   "QUIT",
			Arguments: "",
		}

		if err := rustServer.WriteCommand(command); err != nil && !errors.Is(err, rust_conn.ErrRustServerNotConnected) {
			return err
		}
	}
	return closeErr
}

func (c *Client) Write(message string) error {
	c.writeMutex.Lock()
	defer c.writeMutex.Unlock()

	_, err := c.Conn.Write([]byte(message + "\n"))
	return err
}

type Room struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

var room = Room{
	clients: make(map[string]*Client, config.RoomSize),
}

func (c *Client) SetUsername(username string) string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	if c.State == AUTHENTICATED {
		return responseAlreadyConnected
	}

	if len(room.clients) >= config.RoomSize {
		return responseRoomFull
	}

	if _, ok := room.clients[username]; ok {
		return responseUsernameAlreadyUsed
	}
	c.Username = username
	c.State = AUTHENTICATED
	room.clients[username] = c

	return ""
}

func (c *Client) EraseUsername() {
	room.mutex.Lock()
	if c.State == AUTHENTICATED {
		delete(room.clients, c.Username)
	}
	room.mutex.Unlock()
}

func ConnectedUsernames() []string {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	usernames := make([]string, 0, len(room.clients))
	for username := range room.clients {
		usernames = append(usernames, username)
	}

	return usernames
}

func (c *Client) ReadEvent() string {
	return <-c.eventChan
}

func (c *Client) ReadCommand() string {
	return <-c.commandChan
}

func RouteCommand(username string, command string) bool {
	room.mutex.Lock()
	client, ok := room.clients[strings.ToUpper(username)]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.commandChan <- command
	return true
}

func RouteEvent(username string, event string) bool {
	room.mutex.Lock()
	client, ok := room.clients[strings.ToUpper(username)]
	room.mutex.Unlock()

	if !ok {
		return false
	}

	client.eventChan <- event
	return true
}
