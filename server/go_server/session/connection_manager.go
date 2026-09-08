package session

import (
	"go_server/config"
	serverError "go_server/error"
	"net"
	"sync"
	"time"
)

type ConnectionManager struct {
	mutex                 sync.Mutex
	connections           map[*Client]*time.Timer
	maxConnection         int
	authenticationTimeout time.Duration
	connectionAttempts    map[string]*rateWindow
	lastAttemptCleanup    time.Time
	floodManager          *FloodManager
}

func NewConnectionManager() *ConnectionManager {
	return newConnectionManager(config.MaxConnection, config.AuthenticationTimeout)
}

func newConnectionManager(maxConnection int, authenticationTimeout time.Duration) *ConnectionManager {
	return &ConnectionManager{
		connections:           make(map[*Client]*time.Timer, maxConnection),
		maxConnection:         maxConnection,
		authenticationTimeout: authenticationTimeout,
		connectionAttempts:    make(map[string]*rateWindow),
		floodManager:          NewFloodManager(),
	}
}

func remoteHost(client *Client) string {
	address := client.Conn.RemoteAddr().String()
	host, _, err := net.SplitHostPort(address)
	if err != nil {
		return address
	}
	return host
}

func (manager *ConnectionManager) Subscribe(client *Client) error {
	if manager == nil {
		return serverError.ErrConnectionManagerMissing
	}
	if client == nil || client.Conn == nil {
		return serverError.ErrInvalidConnection
	}

	manager.mutex.Lock()

	host := remoteHost(client)
	if manager.floodManager.IsBanned(host) {
		manager.mutex.Unlock()
		return serverError.ErrIPBanned
	}

	now := time.Now()
	if now.Sub(manager.lastAttemptCleanup) >= config.ConnectionAttemptWindow {
		for host, window := range manager.connectionAttempts {
			if window.expired(now, config.ConnectionAttemptWindow) {
				delete(manager.connectionAttempts, host)
			}
		}
		manager.lastAttemptCleanup = now
	}
	if manager.connectionAttempts[host] == nil {
		manager.connectionAttempts[host] = &rateWindow{}
	}
	if !manager.connectionAttempts[host].allow(now, config.MaxConnectionAttempts, config.ConnectionAttemptWindow) {
		manager.mutex.Unlock()
		manager.registerFlood(host, nil)
		return serverError.ErrRateLimitExceeded
	}

	if _, ok := manager.connections[client]; ok {
		manager.mutex.Unlock()
		return serverError.ErrConnectionAlreadySubscribed
	}
	if len(manager.connections) >= manager.maxConnection {
		manager.mutex.Unlock()
		return serverError.ErrMaxConnection
	}

	manager.connections[client] = time.AfterFunc(manager.authenticationTimeout, func() {
		manager.timeoutUnauthenticated(client)
	})
	manager.mutex.Unlock()

	return nil
}

func (manager *ConnectionManager) RunFloodPointDecay(quit <-chan struct{}) {
	if manager == nil {
		return
	}
	manager.floodManager.RunDecay(quit)
}

func (manager *ConnectionManager) AllowInput(client *Client) bool {
	if manager == nil || client == nil || client.Conn == nil {
		return false
	}

	host := remoteHost(client)
	allowed, banned := manager.floodManager.AllowInput(host)
	if banned {
		manager.disconnectHost(host, client)
	}
	return allowed
}

func (manager *ConnectionManager) registerFlood(host string, ignoredClient *Client) {
	if manager == nil || host == "" {
		return
	}

	if !manager.floodManager.AddFloodPoint(host) {
		return
	}
	manager.disconnectHost(host, ignoredClient)
}

func (manager *ConnectionManager) disconnectHost(host string, ignoredClient *Client) {
	manager.mutex.Lock()
	connections := make([]net.Conn, 0)
	for connectedClient := range manager.connections {
		if connectedClient != ignoredClient && remoteHost(connectedClient) == host {
			connections = append(connections, connectedClient.Conn)
		}
	}
	manager.mutex.Unlock()

	for _, connection := range connections {
		_ = connection.Close()
	}
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
