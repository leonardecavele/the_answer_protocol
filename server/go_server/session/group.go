package session

import (
	"go_server/config"
	"go_server/helper"
	"go_server/protocol"
	"strings"
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

func NewGroup(leader *Client) (*Group, error) {
	group := Group{
		clients: make(map[string]*Client, config.GroupSize),
		invites: make(map[string]time.Time),
		leader:  leader.Username,
	}

	id, err := helper.NewID()
	if err != nil {
		return nil, err
	}
	group.Id = id
	group.clients[leader.Username] = leader
	leader.Group = &group

	return &group, nil
}

func (group *Group) BroadcastEvent(eventBatch protocol.EventBatch) {
	ignored := make(map[string]struct{}, len(eventBatch.IgnoredPlayers))
	for _, username := range eventBatch.IgnoredPlayers {
		username = strings.ToUpper(strings.TrimSpace(username))
		if username != "" {
			ignored[username] = struct{}{}
		}
	}

	target := strings.ToUpper(strings.TrimSpace(eventBatch.Player))
	group.mutex.Lock()
	clients := make([]*Client, 0, len(group.clients))
	if target != "" {
		if _, ok := ignored[target]; !ok {
			if client, ok := group.clients[target]; ok {
				clients = append(clients, client)
			}
		}
	} else {
		for username, client := range group.clients {
			if _, ok := ignored[username]; ok {
				continue
			}
			clients = append(clients, client)
		}
	}
	group.mutex.Unlock()

	for _, client := range clients {
		for _, event := range eventBatch.Events {
			client.eventChan <- event
		}
	}
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
	expiresAt, ok := group.invites[c.Username]
	if !ok || time.Now().After(expiresAt) {
		delete(group.invites, c.Username)
		group.mutex.Unlock()
		return protocol.ResponseNotInvited
	}
	if len(group.clients) >= config.GroupSize {
		group.mutex.Unlock()
		return protocol.ResponseGroupFull
	}
	group.clients[c.Username] = c
	delete(group.invites, c.Username)
	group.mutex.Unlock()

	c.Group = group

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
}
