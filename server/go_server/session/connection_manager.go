package session

import (
	"go_server/config"
	serverError "go_server/error"
	"sync"
	"time"
)

type ConnectionManager struct {
	mutex                 sync.Mutex
	connections           map[*Client]*time.Timer
	maxConnection         int
	authenticationTimeout time.Duration
}

func NewConnectionManager() *ConnectionManager {
	return newConnectionManager(config.MaxConnection, config.AuthenticationTimeout)
}

func newConnectionManager(maxConnection int, authenticationTimeout time.Duration) *ConnectionManager {
	return &ConnectionManager{
		connections:           make(map[*Client]*time.Timer, maxConnection),
		maxConnection:         maxConnection,
		authenticationTimeout: authenticationTimeout,
	}
}

func (manager *ConnectionManager) Subscribe(client *Client) error {
	if manager == nil {
		return serverError.ErrConnectionManagerMissing
	}
	if client == nil || client.Conn == nil {
		return serverError.ErrInvalidConnection
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	if _, ok := manager.connections[client]; ok {
		return serverError.ErrConnectionAlreadySubscribed
	}
	if len(manager.connections) >= manager.maxConnection {
		return serverError.ErrMaxConnection
	}

	manager.connections[client] = time.AfterFunc(manager.authenticationTimeout, func() {
		manager.timeoutUnauthenticated(client)
	})

	return nil
}

func (manager *ConnectionManager) Release(client *Client) {
	if manager == nil || client == nil {
		return
	}

	manager.mutex.Lock()
	timer, ok := manager.connections[client]
	if ok {
		delete(manager.connections, client)
	}
	manager.mutex.Unlock()

	if ok && timer != nil {
		timer.Stop()
	}
}

func (manager *ConnectionManager) Count() int {
	if manager == nil {
		return 0
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	return len(manager.connections)
}

func (manager *ConnectionManager) timeoutUnauthenticated(client *Client) {
	manager.mutex.Lock()
	if _, ok := manager.connections[client]; !ok {
		manager.mutex.Unlock()
		return
	}
	if client.IsAuthenticated() {
		manager.connections[client] = nil
		manager.mutex.Unlock()
		return
	}
	delete(manager.connections, client)
	manager.mutex.Unlock()

	_ = client.Conn.Close()
}
