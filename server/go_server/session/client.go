package session

import (
	"encoding/json"
	"errors"
	"fmt"
	"go_server/config"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/logger"
	"go_server/protocol"
	"net"
	"strings"
	"sync"
	"time"
)

type ClientState string

const (
	CONNECTED     ClientState = "CONNECTED"
	AUTHENTICATED ClientState = "AUTHENTICATED"
)

type Client struct {
	Conn                 net.Conn
	Id                   string
	Username             string
	State                ClientState
	Room                 *Room
	commandChan          chan game_conn.CommandFromGameServer
	eventChan            chan protocol.Event
	connectedAt          time.Time
	stateMutex           sync.RWMutex
	group                *Group
	groupMutex           sync.RWMutex
	writeMutex           sync.Mutex
	disconnectedByServer bool
}

func NewClient(conn net.Conn, room *Room) *Client {
	return &Client{
		Conn:        conn,
		Id:          conn.RemoteAddr().String(),
		State:       CONNECTED,
		Room:        room,
		commandChan: make(chan game_conn.CommandFromGameServer, 64),
		eventChan:   make(chan protocol.Event, 64),
		connectedAt: time.Now(),
	}
}

func (c *Client) connectionInfo() (string, ClientState, time.Time) {
	if c == nil {
		return "", CONNECTED, time.Time{}
	}

	c.stateMutex.RLock()
	defer c.stateMutex.RUnlock()

	return c.Username, c.State, c.connectedAt
}

func (c *Client) Disconnect() error {
	if c == nil || c.Conn == nil {
		return nil
	}

	c.stateMutex.Lock()
	c.disconnectedByServer = true
	c.stateMutex.Unlock()
	return c.Conn.Close()
}

func (c *Client) WasDisconnectedByServer() bool {
	if c == nil {
		return false
	}

	c.stateMutex.RLock()
	defer c.stateMutex.RUnlock()
	return c.disconnectedByServer
}

func (c *Client) DeleteClient(gameServerManager *game_conn.GameServerManager) error {
	username := c.Username
	state := c.GetState()

	group := c.GetGroup()
	if state == AUTHENTICATED && group != nil {
		groupedClients := group.GroupedClients()
		c.QuitGroup()

		for _, groupedClient := range groupedClients {
			if groupedClient == c {
				continue
			}
			c.Room.RouteEvent(groupedClient.Username, protocol.Event{
				EmittedBy: username,
				EventName: "GROUP LEAVE",
			})
		}
	}

	c.Room.DeleteUsername(c)

	if state == AUTHENTICATED {
		c.Room.BroadcastEvent(protocol.EventBatch{
			IgnoredPlayers: []string{username},
			Events: []protocol.Event{
				{
					EmittedBy: username,
					EventName: "QUIT",
				},
			},
		})

		c.Room.BroadcastEvent(protocol.EventBatch{
			Events: []protocol.Event{
				{
					EventName: "STATS",
					Data:      fmt.Sprintf("players=%d", c.Room.Count()),
				},
			},
		})
	}

	closeErr := c.Conn.Close()
	if c.WasDisconnectedByServer() {
		closeErr = nil
	}

	if state == AUTHENTICATED {
		if err := gameServerManager.WriteCommand(game_conn.CommandToGameServer{
			Player:    username,
			Command:   "QUIT",
			Arguments: "",
		}); err != nil && !errors.Is(err, serverError.ErrGameServerNotConnected) {
			return err
		}
	}
	return closeErr
}

func (c *Client) GetGroup() *Group {
	if c == nil {
		return nil
	}

	c.groupMutex.RLock()
	defer c.groupMutex.RUnlock()

	return c.group
}

func (c *Client) GetState() ClientState {
	if c == nil {
		return CONNECTED
	}

	c.stateMutex.RLock()
	defer c.stateMutex.RUnlock()

	return c.State
}

func (c *Client) IsAuthenticated() bool {
	return c.GetState() == AUTHENTICATED
}

func (c *Client) authenticate(username string) {
	c.stateMutex.Lock()
	c.Username = username
	c.State = AUTHENTICATED
	c.stateMutex.Unlock()
}

func (c *Client) rollbackAuthentication() {
	c.stateMutex.Lock()
	c.Username = ""
	c.State = CONNECTED
	c.stateMutex.Unlock()
}

func (c *Client) Write(message string) error {
	c.writeMutex.Lock()
	defer c.writeMutex.Unlock()

	if err := c.Conn.SetWriteDeadline(time.Now().Add(config.TCPWriteTimeout)); err != nil {
		return err
	}

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

func (c *Client) ReadCommandTimeout(timeout time.Duration) (game_conn.CommandFromGameServer, bool) {
	timer := time.NewTimer(timeout)
	defer timer.Stop()

	select {
	case command := <-c.commandChan:
		return command, true
	case <-timer.C:
		return game_conn.CommandFromGameServer{}, false
	}
}

func (c *Client) InSameRoom(clients []*Client, gameServerManager *game_conn.GameServerManager) (bool, error) {
	if c == nil || gameServerManager == nil {
		return false, serverError.ErrGameServerNotConnected
	}

	answer, err := gameServerManager.AskQuestion(game_conn.QuestionToGameServer{
		Question: "ROOM_PLAYERS",
		Data:     c.Username,
	})
	if err != nil {
		return false, err
	}

	var usernames []string
	if err := json.Unmarshal([]byte(answer.Data), &usernames); err != nil {
		return false, err
	}

	roomPlayers := make(map[string]struct{}, len(usernames))
	for _, username := range usernames {
		username = strings.ToUpper(strings.TrimSpace(username))
		if username != "" {
			roomPlayers[username] = struct{}{}
		}
	}

	currentUsername := strings.ToUpper(strings.TrimSpace(c.Username))
	for _, client := range clients {
		if client == nil {
			return false, nil
		}

		username := strings.ToUpper(strings.TrimSpace(client.Username))
		if username == "" {
			return false, nil
		}
		if username == currentUsername {
			continue
		}
		if _, ok := roomPlayers[username]; !ok {
			return false, nil
		}
	}

	return true, nil
}

func (c *Client) SendEvent(event protocol.Event) bool {
	select {
	case c.eventChan <- event:
		return true
	default:
		logger.AppLogger.Error("%s Event dropped: event_name=%s data=%v", c.Id, event.EventName, event.Data)
		return false
	}
}

func (c *Client) SendCommand(command game_conn.CommandFromGameServer) bool {
	select {
	case c.commandChan <- command:
		return true
	default:
		return false
	}
}
