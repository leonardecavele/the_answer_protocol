package session

import (
	"go_server/config"
	"go_server/helper"
	"go_server/protocol"
	"sort"
	"strings"
	"sync"
	"time"
)

type GroupInfo struct {
	ID      string
	Leader  string
	Members []string
}

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
	group.clients[usernameKey(leader.Username)] = leader
	leader.groupMutex.Lock()
	leader.group = &group
	leader.groupMutex.Unlock()

	return &group, nil
}

func (group *Group) BroadcastEvent(eventBatch protocol.EventBatch) {
	ignored := make(map[string]struct{}, len(eventBatch.IgnoredPlayers))
	for _, username := range eventBatch.IgnoredPlayers {
		username = usernameKey(username)
		if username != "" {
			ignored[username] = struct{}{}
		}
	}

	target := usernameKey(eventBatch.Player)
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
			client.SendEvent(event)
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

func (group *Group) Info() (GroupInfo, bool) {
	if group == nil {
		return GroupInfo{}, false
	}

	group.mutex.Lock()
	defer group.mutex.Unlock()
	if group.clients == nil {
		return GroupInfo{}, false
	}

	members := make([]string, 0, len(group.clients))
	for _, client := range group.clients {
		members = append(members, client.Username)
	}
	sort.Strings(members)

	return GroupInfo{
		ID:      group.Id,
		Leader:  group.leader,
		Members: members,
	}, true
}

func (group *Group) Invite(username string) string {
	group.mutex.Lock()
	defer group.mutex.Unlock()

	if group.clients == nil {
		return protocol.ResponseGroupNotFound
	}
	key := usernameKey(username)
	if _, ok := group.clients[key]; ok {
		return protocol.ResponseAlreadyInGroup
	}
	if len(group.clients) >= config.GroupSize {
		return protocol.ResponseGroupFull
	}

	now := time.Now()
	group.deleteExpiredInvites(now)
	group.invites[key] = now.Add(config.GroupInviteTTL)

	return ""
}

func (c *Client) JoinGroup(group *Group) string {
	c.groupMutex.Lock()
	defer c.groupMutex.Unlock()

	if c.group != nil {
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
	key := usernameKey(c.Username)
	expiresAt, ok := group.invites[key]
	if !ok || time.Now().After(expiresAt) {
		delete(group.invites, key)
		group.mutex.Unlock()
		return protocol.ResponseNotInvited
	}
	if len(group.clients) >= config.GroupSize {
		group.mutex.Unlock()
		return protocol.ResponseGroupFull
	}
	group.clients[key] = c
	delete(group.invites, key)
	c.group = group
	group.mutex.Unlock()

	return ""
}

func (group *Group) deleteExpiredInvites(now time.Time) {
	for username, expiresAt := range group.invites {
		if now.After(expiresAt) {
			group.deleteInvite(username)
		}
	}
}

func (group *Group) deleteAllInvites() {
	for username := range group.invites {
		group.deleteInvite(username)
	}
}

func (group *Group) deleteInvite(username string) {
	delete(group.invites, username)
	leader := group.clients[usernameKey(group.leader)]
	if leader != nil && leader.Room != nil {
		leader.Room.RouteEvent(username, protocol.Event{
			EmittedBy: group.leader,
			EventName: "GROUP INVITE",
			Data:      "REMOVED",
		})
	}
}

func (c *Client) QuitGroup() {
	c.groupMutex.Lock()
	group := c.group
	if group == nil {
		c.groupMutex.Unlock()
		return
	}

	group.mutex.Lock()
	if strings.EqualFold(c.Username, group.leader) {
		clients := make([]*Client, 0, len(group.clients))
		for _, client := range group.clients {
			clients = append(clients, client)
		}
		group.deleteAllInvites()
		group.clients = nil
		group.mutex.Unlock()
		c.group = nil
		c.groupMutex.Unlock()

		for _, client := range clients {
			if client != c {
				client.clearGroup(group)
			}
		}
		return
	}

	delete(group.clients, usernameKey(c.Username))
	isEmpty := len(group.clients) == 0
	if isEmpty {
		group.clients = nil
	}
	group.mutex.Unlock()
	c.group = nil
	c.groupMutex.Unlock()
}

func (client *Client) IsLeader() bool {
	group := client.GetGroup()
	if group == nil {
		return false
	}

	return strings.EqualFold(client.Username, group.leader)
}

func (c *Client) clearGroup(group *Group) {
	c.groupMutex.Lock()
	if c.group == group {
		c.group = nil
	}
	c.groupMutex.Unlock()
}
