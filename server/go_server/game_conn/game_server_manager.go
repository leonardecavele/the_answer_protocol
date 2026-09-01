package game_conn

import (
	serverError "go_server/error"
	"go_server/helper"
	"go_server/logger"
	"go_server/protocol"
	"sync"
	"time"
)

type GameServerManager struct {
	mutex           sync.RWMutex
	gameServer      *GameServer
	questionManager *QuestionManager
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

func (manager *GameServerManager) resolveQuestion(answer AnswerFromGameServer) bool {
	manager.mutex.Lock()
	if manager.questionManager == nil {
		manager.questionManager = NewQuestionManager()
	}
	questionManager := manager.questionManager
	manager.mutex.Unlock()

	return questionManager.Resolve(answer)
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

func (manager *GameServerManager) WriteQuestion(question QuestionToGameServer) error {
	gameServer := manager.getGameServer()
	if gameServer == nil {
		return serverError.ErrGameServerNotConnected
	}

	return gameServer.WriteQuestion(question)
}

func (manager *GameServerManager) AskQuestion(question QuestionToGameServer) (AnswerFromGameServer, error) {
	if question.Id == "" {
		id, err := helper.NewID()
		if err != nil {
			return AnswerFromGameServer{}, err
		}
		question.Id = id
	}

	manager.mutex.Lock()
	if manager.questionManager == nil {
		manager.questionManager = NewQuestionManager()
	}
	questionManager := manager.questionManager
	manager.mutex.Unlock()

	answerChan, err := questionManager.Subscribe(question.Id)
	if err != nil {
		return AnswerFromGameServer{}, err
	}
	defer questionManager.Unsubscribe(question.Id)

	if err := manager.WriteQuestion(question); err != nil {
		return AnswerFromGameServer{}, err
	}

	select {
	case answer := <-answerChan:
		return answer, nil
	case <-time.After(5 * time.Second):
		return AnswerFromGameServer{}, serverError.ErrGameServerAnswerTimeout
	}
}

func (manager *GameServerManager) HandleGameServer(
	quit <-chan struct{},
	addr string,
	reconnectPlayers func(*GameServerManager) error,
	routeCommand func(username string, command CommandFromGameServer) bool,
	broadcastEvent func(eventBatch protocol.EventBatch),
) {
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

		gameServer.Read(quit, routeCommand, broadcastEvent, manager.resolveQuestion)
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
