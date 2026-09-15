package session

import (
	"go_server/config"
	serverError "go_server/error"
	"sort"
	"strings"
	"sync"
	"time"
	"unicode"
	"unicode/utf8"
)

type ClientConnectionInfo struct {
	Username     string
	IP           string
	State        ClientState
	ConnectedFor time.Duration
}

type ClientConnectionManager struct {
	mutex                 sync.Mutex
	connections           map[*Client]*time.Timer
	maxConnection         int
	authenticationTimeout time.Duration
	connectionAttempts    map[string]*rateWindow
	lastAttemptCleanup    time.Time
	floodManager          *FloodManager
}

func NewClientConnectionManager() *ClientConnectionManager {
	return newClientConnectionManager(config.MaxConnection, config.AuthenticationTimeout)
}

func newClientConnectionManager(maxConnection int, authenticationTimeout time.Duration) *ClientConnectionManager {
	return &ClientConnectionManager{
		connections:           make(map[*Client]*time.Timer, maxConnection),
		maxConnection:         maxConnection,
		authenticationTimeout: authenticationTimeout,
		connectionAttempts:    make(map[string]*rateWindow),
		floodManager:          NewFloodManager(),
	}
}

func (manager *ClientConnectionManager) Subscribe(client *Client) error {
	if manager == nil {
		return serverError.ErrClientConnectionManagerMissing
	}
	if client == nil || client.Conn == nil {
		return serverError.ErrInvalidConnection
	}

	manager.mutex.Lock()

	ip := clientIP(client)
	if manager.floodManager.IsBanned(ip) {
		manager.mutex.Unlock()
		return serverError.ErrIPBanned
	}

	now := time.Now()
	if now.Sub(manager.lastAttemptCleanup) >= config.ConnectionAttemptWindow {
		for ip, window := range manager.connectionAttempts {
			if window.isExpired(now, config.ConnectionAttemptWindow) {
				delete(manager.connectionAttempts, ip)
			}
		}
		manager.lastAttemptCleanup = now
	}
	if manager.connectionAttempts[ip] == nil {
		manager.connectionAttempts[ip] = &rateWindow{}
	}
	if !manager.connectionAttempts[ip].allow(now, config.MaxConnectionAttempts, config.ConnectionAttemptWindow) {
		manager.mutex.Unlock()
		manager.registerFlood(ip, nil)
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

func (manager *ClientConnectionManager) Clients() []ClientConnectionInfo {
	if manager == nil {
		return nil
	}

	now := time.Now()
	manager.mutex.Lock()
	clients := make([]ClientConnectionInfo, 0, len(manager.connections))
	for client := range manager.connections {
		username, state, connectedAt := client.connectionInfo()
		clients = append(clients, ClientConnectionInfo{
			Username:     username,
			IP:           clientIP(client),
			State:        state,
			ConnectedFor: now.Sub(connectedAt).Round(time.Second),
		})
	}
	manager.mutex.Unlock()

	sort.Slice(clients, func(i, j int) bool {
		if clients[i].Username == clients[j].Username {
			return clients[i].IP < clients[j].IP
		}
		return clients[i].Username < clients[j].Username
	})
	return clients
}

func (manager *ClientConnectionManager) IsInputValid(input string) bool {
	if manager == nil || !utf8.ValidString(input) || !strings.HasSuffix(input, "\n") {
		return false
	}

	input = strings.TrimSuffix(input, "\n")
	input = strings.TrimSuffix(input, "\r")

	return strings.IndexFunc(input, unicode.IsControl) == -1
}

func (manager *ClientConnectionManager) Release(client *Client) {
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

func (manager *ClientConnectionManager) Count() int {
	if manager == nil {
		return 0
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	return len(manager.connections)
}

func (manager *ClientConnectionManager) timeoutUnauthenticated(client *Client) {
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

	_ = client.Disconnect()
}
