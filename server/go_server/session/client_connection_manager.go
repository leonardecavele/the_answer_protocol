package session

import (
	"go_server/config"
	serverError "go_server/error"
	"go_server/helper"
	"go_server/logger"
	"net"
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

func remoteHost(client *Client) string {
	address := client.Conn.RemoteAddr().String()
	host, _, err := net.SplitHostPort(address)
	if err != nil {
		return address
	}
	if normalizedHost := helper.NormalizeIP(host); normalizedHost != "" {
		return normalizedHost
	}
	return host
}

func (manager *ClientConnectionManager) Subscribe(client *Client) error {
	if manager == nil {
		return serverError.ErrClientConnectionManagerMissing
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
			if window.isExpired(now, config.ConnectionAttemptWindow) {
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

func (manager *ClientConnectionManager) RunFloodPointDecay(quit <-chan struct{}) {
	if manager == nil {
		return
	}
	manager.floodManager.RunDecay(quit)
}

func (manager *ClientConnectionManager) BanIP(ip string) bool {
	if manager == nil {
		return false
	}

	normalizedHost := helper.NormalizeIP(ip)
	if normalizedHost == "" {
		return false
	}

	manager.floodManager.BanIP(normalizedHost)
	manager.disconnectHost(normalizedHost, nil)
	return true
}

func (manager *ClientConnectionManager) UnbanIP(ip string) bool {
	if manager == nil {
		return false
	}

	normalizedHost := helper.NormalizeIP(ip)
	if normalizedHost == "" {
		return false
	}

	manager.floodManager.ClearIP(normalizedHost)

	manager.mutex.Lock()
	delete(manager.connectionAttempts, normalizedHost)
	manager.mutex.Unlock()

	return true
}

func (manager *ClientConnectionManager) BannedIPs() []IPFloodInfo {
	if manager == nil {
		return nil
	}
	return manager.floodManager.BannedIPs()
}

func (manager *ClientConnectionManager) FloodInfo(ip string) (IPFloodInfo, bool) {
	if manager == nil {
		return IPFloodInfo{}, false
	}

	normalizedHost := helper.NormalizeIP(ip)
	if normalizedHost == "" {
		return IPFloodInfo{}, false
	}
	return manager.floodManager.Info(normalizedHost), true
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
			IP:           remoteHost(client),
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

func (manager *ClientConnectionManager) AllowInput(client *Client) bool {
	if manager == nil || client == nil || client.Conn == nil {
		return false
	}

	host := remoteHost(client)
	allowed, banned := manager.floodManager.AllowInput(host)
	if !allowed {
		manager.logFlood(host, "client_input")
	}
	if banned {
		manager.disconnectHost(host, client)
	}
	return allowed
}

func (manager *ClientConnectionManager) IsInputValid(input string) bool {
	if manager == nil || !utf8.ValidString(input) || !strings.HasSuffix(input, "\n") {
		return false
	}

	input = strings.TrimSuffix(input, "\n")
	input = strings.TrimSuffix(input, "\r")

	return strings.IndexFunc(input, unicode.IsControl) == -1
}

func (manager *ClientConnectionManager) registerFlood(host string, ignoredClient *Client) {
	if manager == nil || host == "" {
		return
	}

	banned := manager.floodManager.AddFloodPoint(host)
	manager.logFlood(host, "connection_attempt")
	if !banned {
		return
	}
	manager.disconnectHost(host, ignoredClient)
}

func (manager *ClientConnectionManager) logFlood(host string, source string) {
	info := manager.floodManager.Info(host)
	logger.AppLogger.Warn(
		"Flood detected: ip=%s source=%s points=%d/%d banned=%t",
		info.IP,
		source,
		info.Points,
		info.MaxPoints,
		info.Banned,
	)
}

func (manager *ClientConnectionManager) disconnectHost(host string, ignoredClient *Client) {
	manager.mutex.Lock()
	clients := make([]*Client, 0)
	for connectedClient := range manager.connections {
		if connectedClient != ignoredClient && remoteHost(connectedClient) == host {
			clients = append(clients, connectedClient)
		}
	}
	manager.mutex.Unlock()

	for _, client := range clients {
		_ = client.Disconnect()
	}
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
