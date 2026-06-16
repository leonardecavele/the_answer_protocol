package client_conn

import (
	"go_server/config"
	"go_server/game_conn"
	"sync"
)

type Group struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

func NewGroup() *Group {
	return &Group{
		clients: make(map[string]*Client, config.GroupSize),
	}
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
