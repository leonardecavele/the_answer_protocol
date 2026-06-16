package game_conn

import (
	"go_server/protocol"
	"strconv"
	"sync"

	"go_server/config"
	serverError "go_server/error"
	"go_server/logger"
)

type GameServerManager struct {
	mutex      sync.RWMutex
	gameServer *GameServer
}

func (manager *GameServerManager) getGameServer() *GameServer {
	if manager == nil {
		return nil
	}

	manager.mutex.RLock()
	defer manager.mutex.RUnlock()

	return manager.gameServer
}

func (manager *GameServerManager) SetGameServer(gameServer *GameServer) {
	manager.mutex.Lock()
	oldGameServer := manager.gameServer
	manager.gameServer = gameServer
	manager.mutex.Unlock()

	if oldGameServer != nil && oldGameServer != gameServer {
		_ = oldGameServer.Close()
	}
}

func (manager *GameServerManager) ClearServer(gameServer *GameServer) {
	manager.mutex.Lock()
	if manager.gameServer == gameServer {
		manager.gameServer = nil
	}
	manager.mutex.Unlock()
}

func (manager *GameServerManager) IsConnected() bool {
	return manager.getGameServer() != nil
}

func (manager *GameServerManager) Write(message string) error {
	gameServer := manager.getGameServer()
	if gameServer == nil {
		return serverError.ErrGameServerNotConnected
	}

	return gameServer.Write(message)
}

func (manager *GameServerManager) WriteCommand(command any) error {
	gameServer := manager.getGameServer()
	if gameServer == nil {
		return serverError.ErrGameServerNotConnected
	}

	return gameServer.WriteCommand(command)
}

func (manager *GameServerManager) HandleGameServer(
	quit <-chan struct{},
	reconnectPlayers func(*GameServerManager) error,
	routeCommand func(username string, command CommandFromGameServer) bool,
	routeEvent func(username string, event protocol.Event) bool,
) {
	addr := config.GameServerIP + ":" + strconv.Itoa(config.GameServerPort)

	for {
		select {
		case <-quit:
			return
		default:
		}

		gameServer := ConnectToGameServer(addr, quit)
		if gameServer == nil {
			return
		}

		manager.SetGameServer(gameServer)

		if reconnectPlayers != nil {
			if err := reconnectPlayers(manager); err != nil {
				logger.AppLogger.Error("Game server players reconnect error: %v", err)
				manager.ClearServer(gameServer)
				_ = gameServer.Close()
				continue
			}
		}

		stopClosingGameServer := make(chan struct{})
		go func() {
			select {
			case <-quit:
				_ = gameServer.Close()
			case <-stopClosingGameServer:
			}
		}()

		gameServer.Read(quit, routeCommand, routeEvent)
		close(stopClosingGameServer)

		manager.ClearServer(gameServer)
		_ = gameServer.Close()

		select {
		case <-quit:
			return
		default:
		}
		logger.AppLogger.Info("Waiting for Game server reconnect")
	}
}
