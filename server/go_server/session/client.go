package session

import (
	"errors"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/protocol"
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
	Group       *Group
	Room        *Room
	commandChan chan game_conn.CommandFromGameServer
	eventChan   chan protocol.Event
	writeMutex  sync.Mutex
}

func NewClient(conn net.Conn, room *Room) *Client {
	return &Client{
		Conn:        conn,
		Id:          conn.RemoteAddr().String(),
		State:       CONNECTED,
		Room:        room,
		commandChan: make(chan game_conn.CommandFromGameServer, 16),
		eventChan:   make(chan protocol.Event, 16),
	}
}

func (c *Client) DeleteClient(gameServer *game_conn.GameServerManager) error {
	username := c.Username
	state := c.State

	if state == AUTHENTICATED && c.Group != nil {
		c.QuitGroup()
	}

	c.Room.BroadcastEvent(protocol.Event{
		IgnoredPlayers: []string{username},
		EmittedBy:      username,
		EventName:      "QUIT",
	})

	c.Room.DeleteUsername(c)
	closeErr := c.Conn.Close()

	if state == AUTHENTICATED {
		if err := gameServer.WriteCommand(game_conn.CommandToGameServer{
			Player:    username,
			Command:   "QUIT",
			Arguments: "",
		}); err != nil && !errors.Is(err, serverError.ErrGameServerNotConnected) {
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

func (c *Client) ReadEvent() protocol.Event {
	return <-c.eventChan
}

func (c *Client) Events() <-chan protocol.Event {
	return c.eventChan
}

func (c *Client) ReadCommand() game_conn.CommandFromGameServer {
	return <-c.commandChan
}
