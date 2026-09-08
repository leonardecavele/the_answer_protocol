package session

import (
	"go_server/config"
	"sync"
	"time"
)

type FloodManager struct {
	mutex         sync.Mutex
	pointsByIP    map[string]int
	inputsByIP    map[string]*rateWindow
	maxPoints     int
	inputLimit    int
	inputWindow   time.Duration
	decayInterval time.Duration
}

func NewFloodManager() *FloodManager {
	return newFloodManager(config.MaxFloodPoints, config.FloodPointDecayInterval)
}

func newFloodManager(maxPoints int, decayInterval time.Duration) *FloodManager {
	return &FloodManager{
		pointsByIP:    make(map[string]int),
		inputsByIP:    make(map[string]*rateWindow),
		maxPoints:     maxPoints,
		inputLimit:    config.MaxCommandsPerWindow,
		inputWindow:   config.CommandRateWindow,
		decayInterval: decayInterval,
	}
}

func (manager *FloodManager) AllowInput(ip string) (bool, bool) {
	return manager.allowInput(ip, time.Now())
}

func (manager *FloodManager) allowInput(ip string, now time.Time) (bool, bool) {
	if manager == nil || ip == "" {
		return false, false
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	if manager.pointsByIP[ip] > manager.maxPoints {
		return false, true
	}
	if manager.inputsByIP[ip] == nil {
		manager.inputsByIP[ip] = &rateWindow{}
	}
	if manager.inputsByIP[ip].allow(now, manager.inputLimit, manager.inputWindow) {
		return true, false
	}

	manager.pointsByIP[ip]++
	return false, manager.pointsByIP[ip] > manager.maxPoints
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

	now := time.Now()
	for ip, window := range manager.inputsByIP {
		if window.expired(now, manager.inputWindow) {
			delete(manager.inputsByIP, ip)
		}
	}
	for ip, points := range manager.pointsByIP {
		if points <= 1 {
			delete(manager.pointsByIP, ip)
			continue
		}
		manager.pointsByIP[ip]--
	}
}
