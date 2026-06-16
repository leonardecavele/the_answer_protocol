package session

import (
	"crypto/rand"
	"encoding/hex"
	"go_server/config"
	"go_server/game_conn"
	"go_server/protocol"
	"sync"
)

type Group struct {
	clients map[string]*Client
	mutex   sync.Mutex
	Id      string
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
	group.Id = id

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
	group.clients[c.Username] = c
	group.mutex.Unlock()

	c.Group = group
	group.BroadcastEvent(game_conn.EventFromGameServer{
		Player:    c.Username,
		EventName: "GROUP JOIN",
		Data:      c.Username,
	})

	return ""
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

	c.Group = nil
	if !isEmpty {
		group.BroadcastEvent(game_conn.EventFromGameServer{
			Player:    c.Username,
			EventName: "GROUP LEAVE",
			Data:      c.Username,
		})
	}
}
