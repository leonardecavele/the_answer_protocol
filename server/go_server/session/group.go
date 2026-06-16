package session

import (
	"crypto/rand"
	"encoding/hex"
	"go_server/config"
	"go_server/game_conn"
	"go_server/protocol"
	"sync"
	"time"
)

type Group struct {
	clients map[string]*Client
	invites map[string]time.Time
	mutex   sync.Mutex
	Id      string
	leader  string
}

func NewGroup(leader string) (*Group, error) {
	group := Group{
		clients: make(map[string]*Client, config.GroupSize),
		invites: make(map[string]time.Time),
		leader:  leader,
	}

	id, err := group.newId()
	if err != nil {
		return nil, err
	}
	group.Id = id

	return &group, nil
}

func (group *Group) BroadcastEvent(event game_conn.Event) {
	group.mutex.Lock()
	clients := make([]*Client, 0, len(group.clients))
	for _, client := range group.clients {
		clients = append(clients, client)
	}
	group.mutex.Unlock()

	for _, client := range clients {
		client.eventChan <- event
	}
}

func (group *Group) BroadcastEventExcept(event game_conn.Event, excludedClient *Client) {
	group.mutex.Lock()
	clients := make([]*Client, 0, len(group.clients))
	for _, client := range group.clients {
		if client == excludedClient {
			continue
		}
		clients = append(clients, client)
	}
	group.mutex.Unlock()

	for _, client := range clients {
		client.eventChan <- event
	}
}

func (group *Group) newId() (string, error) {
	group.mutex.Lock()
	defer group.mutex.Unlock()

	bytes := make([]byte, 16)

	if _, err := rand.Read(bytes); err != nil {
		return "", err
	}

	return hex.EncodeToString(bytes), nil
}

func (group *Group) GroupedClients() []*Client {
	group.mutex.Lock()
	defer group.mutex.Unlock()

	clients := make([]*Client, 0, len(group.clients))
	for _, client := range group.clients {
		clients = append(clients, client)
	}

	return clients
}

func (group *Group) Invite(username string) string {
	group.mutex.Lock()
	defer group.mutex.Unlock()

	if group.clients == nil {
		return protocol.ResponseGroupNotFound
	}
	if _, ok := group.clients[username]; ok {
		return protocol.ResponseAlreadyInGroup
	}
	if len(group.clients) >= config.GroupSize {
		return protocol.ResponseGroupFull
	}

	now := time.Now()
	group.deleteExpiredInvites(now)
	group.invites[username] = now.Add(config.GroupInviteTTL)

	return ""
}

func (c *Client) JoinGroup(group *Group) string {
	return c.joinGroup(group, false)
}

func (c *Client) JoinInvitedGroup(group *Group) string {
	return c.joinGroup(group, true)
}

func (c *Client) joinGroup(group *Group, requireInvite bool) string {
	if c.Group != nil {
		return protocol.ResponseAlreadyInGroup
	}
	if group == nil {
		return protocol.ResponseGroupNotFound
	}

	group.mutex.Lock()
	if group.clients == nil {
		group.mutex.Unlock()
		return protocol.ResponseGroupNotFound
	}
	if requireInvite {
		expiresAt, ok := group.invites[c.Username]
		if !ok || time.Now().After(expiresAt) {
			delete(group.invites, c.Username)
			group.mutex.Unlock()
			return protocol.ResponseNotInvited
		}
	}
	if len(group.clients) >= config.GroupSize {
		group.mutex.Unlock()
		return protocol.ResponseGroupFull
	}
	group.clients[c.Username] = c
	delete(group.invites, c.Username)
	group.mutex.Unlock()

	c.Group = group
	group.BroadcastEvent(game_conn.Event{
		Player:    c.Username,
		EventName: "GROUP JOIN",
		Data:      c.Username,
	})

	return ""
}

func (group *Group) deleteExpiredInvites(now time.Time) {
	for username, expiresAt := range group.invites {
		if now.After(expiresAt) {
			delete(group.invites, username)
		}
	}
}

func (c *Client) QuitGroup() {
	group := c.Group
	if group == nil {
		return
	}

	group.mutex.Lock()
	if c.Username == group.leader {
		clients := make([]*Client, 0, len(group.clients))
		for _, client := range group.clients {
			clients = append(clients, client)
		}
		group.clients = nil
		group.mutex.Unlock()

		for _, client := range clients {
			client.Group = nil
			if client != c {
				client.eventChan <- game_conn.Event{
					Player:    c.Username,
					EventName: "GROUP LEAVE",
					Data:      c.Username,
				}
			}
		}
		return
	}

	delete(group.clients, c.Username)
	isEmpty := len(group.clients) == 0
	if isEmpty {
		group.clients = nil
	}
	group.mutex.Unlock()

	c.Group = nil
	if !isEmpty {
		group.BroadcastEvent(game_conn.Event{
			Player:    c.Username,
			EventName: "GROUP LEAVE",
			Data:      c.Username,
		})
	}
}
