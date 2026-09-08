package session

import (
	"go_server/config"
	"sync"
	"time"
)

type FloodManager struct {
	mutex         sync.Mutex
	pointsByIP    map[string]int
	maxPoints     int
	decayInterval time.Duration
}

func NewFloodManager() *FloodManager {
	return newFloodManager(config.MaxFloodPoints, config.FloodPointDecayInterval)
}

func newFloodManager(maxPoints int, decayInterval time.Duration) *FloodManager {
	return &FloodManager{
		pointsByIP:    make(map[string]int),
		maxPoints:     maxPoints,
		decayInterval: decayInterval,
	}
}

func (manager *FloodManager) AddFloodPoint(ip string) bool {
	if manager == nil || ip == "" {
		return false
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	manager.pointsByIP[ip]++
	return manager.pointsByIP[ip] > manager.maxPoints
}

func (manager *FloodManager) IsBanned(ip string) bool {
	if manager == nil || ip == "" {
		return false
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	return manager.pointsByIP[ip] > manager.maxPoints
}

func (manager *FloodManager) RunDecay(quit <-chan struct{}) {
	if manager == nil || manager.decayInterval <= 0 {
		return
	}

	ticker := time.NewTicker(manager.decayInterval)
	defer ticker.Stop()

	for {
		select {
		case <-quit:
			return
		case <-ticker.C:
			manager.decreaseFloodPoints()
		}
	}
}

func (manager *FloodManager) decreaseFloodPoints() {
	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	for ip, points := range manager.pointsByIP {
		if points <= 1 {
			delete(manager.pointsByIP, ip)
			continue
		}
		manager.pointsByIP[ip]--
	}
}
