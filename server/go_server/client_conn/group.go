package client_conn

import (
	"go_server/game_conn"
	"sync"
)

type Group struct {
	clients map[string]*Client
	mutex   sync.Mutex
}

func (g *Group) BroadcastEvent(event game_conn.EventFromGameServer) {
	return
}
