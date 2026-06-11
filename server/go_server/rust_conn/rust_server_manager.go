package rust_conn

import (
	"strconv"
	"sync"

	"go_server/config"
	"go_server/logger"
)

type RustServerManager struct {
	mutex      sync.RWMutex
	rustServer *RustServer
}

func (manager *RustServerManager) getRustServer() *RustServer {
	if manager == nil {
		return nil
	}

	manager.mutex.RLock()
	defer manager.mutex.RUnlock()

	return manager.rustServer
}

func (manager *RustServerManager) SetRustServer(rustServer *RustServer) {
	manager.mutex.Lock()
	oldRustServer := manager.rustServer
	manager.rustServer = rustServer
	manager.mutex.Unlock()

	if oldRustServer != nil && oldRustServer != rustServer {
		_ = oldRustServer.Close()
	}
}

func (manager *RustServerManager) ClearServer(rustServer *RustServer) {
	manager.mutex.Lock()
	if manager.rustServer == rustServer {
		manager.rustServer = nil
	}
	manager.mutex.Unlock()
}

func (manager *RustServerManager) IsConnected() bool {
	return manager.getRustServer() != nil
}

func (manager *RustServerManager) Write(message string) error {
	rustServer := manager.getRustServer()
	if rustServer == nil {
		return ErrRustServerNotConnected
	}

	return rustServer.Write(message)
}

func (manager *RustServerManager) WriteCommand(command any) error {
	rustServer := manager.getRustServer()
	if rustServer == nil {
		return ErrRustServerNotConnected
	}

	return rustServer.WriteCommand(command)
}

func (manager *RustServerManager) HandleRustServer(
	quit <-chan struct{},
	reconnectPlayers func(*RustServerManager) error,
	routeCommand func(username string, command string) bool,
	routeEvent func(username string, event string) bool,
) {
	addr := config.RustServerIP + ":" + strconv.Itoa(config.RustServerPort)

	for {
		select {
		case <-quit:
			return
		default:
		}

		rustServer := ConnectToRust(addr, quit)
		if rustServer == nil {
			return
		}

		manager.SetRustServer(rustServer)

		if reconnectPlayers != nil {
			if err := reconnectPlayers(manager); err != nil {
				logger.AppLogger.Error("Rust players reconnect error: %v", err)
				manager.ClearServer(rustServer)
				_ = rustServer.Close()
				continue
			}
		}

		stopClosingRust := make(chan struct{})
		go func() {
			select {
			case <-quit:
				_ = rustServer.Close()
			case <-stopClosingRust:
			}
		}()

		rustServer.Read(quit, routeCommand, routeEvent)
		close(stopClosingRust)

		manager.ClearServer(rustServer)
		_ = rustServer.Close()

		select {
		case <-quit:
			return
		default:
		}
		logger.AppLogger.Info("Waiting for Rust server reconnect")
	}
}
