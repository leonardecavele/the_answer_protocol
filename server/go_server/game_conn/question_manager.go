package game_conn

import (
	serverError "go_server/error"
	"sync"
)

type QuestionManager struct {
	mutex       sync.Mutex
	subscribers map[string]chan AnswerFromGameServer
}

func NewQuestionManager() *QuestionManager {
	return &QuestionManager{
		subscribers: make(map[string]chan AnswerFromGameServer),
	}
}

func (manager *QuestionManager) Subscribe(id string) (<-chan AnswerFromGameServer, error) {
	if manager == nil {
		return nil, serverError.ErrQuestionManagerMissing
	}
	if id == "" {
		return nil, serverError.ErrInvalidQuestionID
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	if _, ok := manager.subscribers[id]; ok {
		return nil, serverError.ErrQuestionSubscribed
	}

	answerChan := make(chan AnswerFromGameServer, 1)
	manager.subscribers[id] = answerChan

	return answerChan, nil
}

func (manager *QuestionManager) Unsubscribe(id string) {
	if manager == nil || id == "" {
		return
	}

	manager.mutex.Lock()
	delete(manager.subscribers, id)
	manager.mutex.Unlock()
}

func (manager *QuestionManager) Resolve(answer AnswerFromGameServer) bool {
	if manager == nil || answer.Id == "" {
		return false
	}

	manager.mutex.Lock()
	answerChan, ok := manager.subscribers[answer.Id]
	if ok {
		delete(manager.subscribers, answer.Id)
	}
	manager.mutex.Unlock()

	if !ok {
		return false
	}

	answerChan <- answer
	return true
}
