package client_conn

import (
	"errors"
	"go_server/game_conn"
	"net"
	"sync"
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
	room        *Room
	commandChan chan string
	eventChan   chan game_conn.EventFromGameServer
	writeMutex  sync.Mutex
}

func NewClient(conn net.Conn, room *Room) *Client {
	return &Client{
		Conn:        conn,
		Id:          conn.RemoteAddr().String(),
		State:       CONNECTED,
		room:        room,
		commandChan: make(chan string, 16),
		eventChan:   make(chan game_conn.EventFromGameServer, 16),
	}
}

func (c *Client) EraseClient(gameServer *game_conn.GameServerManager) error {
	username := c.Username
	state := c.State

	c.EraseUsername()
	closeErr := c.Conn.Close()

	if state == AUTHENTICATED {
		command := game_conn.CommandToGameServer{
			Player:    username,
			Command:   "QUIT",
			Arguments: "",
		}

		if err := gameServer.WriteCommand(command); err != nil && !errors.Is(err, game_conn.ErrGameServerNotConnected) {
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

func (c *Client) SetUsername(username string) string {
	return c.room.SetUsername(c, username)
}

func (c *Client) EraseUsername() {
	c.room.EraseUsername(c)
}

func (c *Client) ReadEvent() game_conn.EventFromGameServer {
	return <-c.eventChan
}

func (c *Client) ReadCommand() string {
	return <-c.commandChan
}
