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
	group       *Group
	room        *Room
	commandChan chan game_conn.CommandFromGameServer
	eventChan   chan game_conn.EventFromGameServer
	writeMutex  sync.Mutex
}

func NewClient(conn net.Conn, room *Room) *Client {
	return &Client{
		Conn:        conn,
		Id:          conn.RemoteAddr().String(),
		State:       CONNECTED,
		room:        room,
		commandChan: make(chan game_conn.CommandFromGameServer, 16),
		eventChan:   make(chan game_conn.EventFromGameServer, 16),
	}
}

func (c *Client) DeleteClient(gameServer *game_conn.GameServerManager) error {
	username := c.Username
	state := c.State

	if state == AUTHENTICATED && c.group != nil {
		c.QuitGroup()
	}
	c.room.DeleteUsername(c)
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

func (c *Client) JoinGroup(group *Group) string {
	if c.group != nil {
		return responseAlreadyInGroup
	}
	if group == nil {
		return responseGroupNotFound
	}

	group.mutex.Lock()
	if group.clients == nil {
		group.mutex.Unlock()
		return responseGroupNotFound
	}
	group.clients[c.Username] = c
	group.mutex.Unlock()

	c.group = group
	group.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    c.Username,
		EventName: "GROUP JOIN",
		Data:      c.Username,
	})

	return ""
}

func (c *Client) QuitGroup() {
	group := c.group
	if group == nil {
		return
	}

	group.mutex.Lock()
	delete(group.clients, c.Username)
	isEmpty := len(group.clients) == 0
	if isEmpty {
		group.clients = nil
	}
	group.mutex.Unlock()

	c.group = nil
	if !isEmpty {
		group.BroadcastEvent(game_conn.EventFromGameServer{
			Player:    c.Username,
			EventName: "GROUP LEAVE",
			Data:      c.Username,
		})
	}
}

func (c *Client) ReadEvent() game_conn.EventFromGameServer {
	return <-c.eventChan
}

func (c *Client) ReadCommand() game_conn.CommandFromGameServer {
	return <-c.commandChan
}
