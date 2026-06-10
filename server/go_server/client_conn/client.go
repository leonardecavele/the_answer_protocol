package client_conn

import (
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

func (c *Client) EraseClient() {
	c.EraseUsername()
	c.Conn.Close()
}

func (c *Client) Write(message string) error {
	c.writeMutex.Lock()
	defer c.writeMutex.Unlock()

	_, err := c.Conn.Write([]byte(message))
	return err
}

type Room struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

var room = Room{
	clients: make(map[string]*Client, config.RoomSize),
}

func (c *Client) SetUsername(username string) error {
	room.mutex.Lock()
	defer room.mutex.Unlock()

	if c.State == AUTHENTICATED {
		return errClientAlreadyHasUsername
	}

	if len(room.clients) >= config.RoomSize {
		return errRoomFull
	}

	if _, ok := room.clients[username]; ok {
		return errUsernameAlreadyUsed
	}
	c.Username = username
	c.State = AUTHENTICATED
	room.clients[username] = c

	return nil
}

func (c *Client) EraseUsername() {
	room.mutex.Lock()
	if c.State == AUTHENTICATED {
		delete(room.clients, c.Username)
	}
	c.Username = ""
	c.State = CONNECTED
	room.mutex.Unlock()
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
