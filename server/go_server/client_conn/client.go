package client_conn

import (
	"net"
	"sync"
)

import (
	"go_server/config"
)

type Client struct {
	Conn     net.Conn
	Id       string
	Username string
}

func NewClient(conn net.Conn) *Client {
	return &Client{
		Conn: conn,
		Id:   conn.RemoteAddr().String(),
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

	if c.Username != "" {
		return errClientAlreadyHasUsername
	}

	if len(room.clients) >= config.RoomSize {
		return errRoomFull
	}

	if _, ok := room.clients[username]; ok {
		return errUsernameAlreadyUsed
	}

	c.Username = username
	room.clients[username] = c

	return nil
}

func (c *Client) EraseUsername() {
	room.mutex.Lock()
	if c.Username != "" {
		delete(room.clients, c.Username)
	}
	room.mutex.Unlock()
}
