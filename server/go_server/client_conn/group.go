package client_conn

import (
	"crypto/rand"
	"encoding/hex"
	"go_server/config"
	"go_server/game_conn"
	"sync"
)

type Group struct {
	clients map[string]*Client
	mutex   sync.Mutex
	id      string
	leader  string
}

func NewGroup(leader string) (*Group, error) {
	group := Group{
		clients: make(map[string]*Client, config.GroupSize),
		leader:  leader,
	}

	id, err := group.newId()
	if err != nil {
		return nil, err
	}
	group.id = id

	return &group, nil
}

func (group *Group) BroadcastEvent(event game_conn.EventFromGameServer) {
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
	if c.Username == group.leader {
		clients := c.group.GroupedClients()
		group.clients = nil
		group.mutex.Unlock()

		for _, client := range clients {
			client.group = nil
			if client != c {
				client.eventChan <- game_conn.EventFromGameServer{
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

	c.group = nil
	if !isEmpty {
		group.BroadcastEvent(game_conn.EventFromGameServer{
			Player:    c.Username,
			EventName: "GROUP LEAVE",
			Data:      c.Username,
		})
	}
}
