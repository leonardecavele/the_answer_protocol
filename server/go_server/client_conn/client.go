package client_conn

import (
	"net"
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
	Conn     net.Conn
	Id       string
	Username string
	State    ClientState
}

func NewClient(conn net.Conn) *Client {
	return &Client{
		Conn:  conn,
		Id:    conn.RemoteAddr().String(),
		State: CONNECTED,
	}
}

func (c *Client) EraseClient() {
	c.EraseUsername()
	c.Conn.Close()
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
