package session

import (
	"go_server/helper"
	"go_server/logger"
)

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

	normalizedIP := helper.NormalizeIP(ip)
	if normalizedIP == "" {
		return false
	}

	manager.floodManager.BanIP(normalizedIP)
	manager.disconnectIP(normalizedIP, nil)
	return true
}

func (manager *ClientConnectionManager) UnbanIP(ip string) bool {
	if manager == nil {
		return false
	}

	normalizedIP := helper.NormalizeIP(ip)
	if normalizedIP == "" {
		return false
	}

	manager.floodManager.ClearIP(normalizedIP)

	manager.mutex.Lock()
	delete(manager.connectionAttempts, normalizedIP)
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

	normalizedIP := helper.NormalizeIP(ip)
	if normalizedIP == "" {
		return IPFloodInfo{}, false
	}
	return manager.floodManager.Info(normalizedIP), true
}

func (manager *ClientConnectionManager) AllowInput(client *Client) bool {
	if manager == nil || client == nil || client.Conn == nil {
		return false
	}

	ip := clientIP(client)
	allowed, banned := manager.floodManager.AllowInput(ip)
	if !allowed {
		manager.logFlood(ip, "client_input")
	}
	if banned {
		manager.disconnectIP(ip, client)
	}
	return allowed
}

func (manager *ClientConnectionManager) registerFlood(ip string, ignoredClient *Client) {
	if manager == nil || ip == "" {
		return
	}

	banned := manager.floodManager.AddFloodPoint(ip)
	manager.logFlood(ip, "connection_attempt")
	if !banned {
		return
	}
	manager.disconnectIP(ip, ignoredClient)
}

func (manager *ClientConnectionManager) logFlood(ip string, source string) {
	info := manager.floodManager.Info(ip)
	logger.AppLogger.Warn(
		"Flood detected: ip=%s source=%s points=%d/%d banned=%t",
		info.IP,
		source,
		info.Points,
		info.MaxPoints,
		info.Banned,
	)
}

func (manager *ClientConnectionManager) disconnectIP(ip string, ignoredClient *Client) {
	manager.mutex.Lock()
	clients := make([]*Client, 0)
	for connectedClient := range manager.connections {
		if connectedClient != ignoredClient && clientIP(connectedClient) == ip {
			clients = append(clients, connectedClient)
		}
	}
	manager.mutex.Unlock()

	for _, client := range clients {
		_ = client.Disconnect()
	}
}
