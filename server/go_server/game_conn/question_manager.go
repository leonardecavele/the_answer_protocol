package game_conn

import (
	"errors"
	"sync"
)

var (
	ErrInvalidQuestionID          = errors.New("invalid question id")
	ErrQuestionAlreadySubscribed  = errors.New("question already subscribed")
	ErrQuestionManagerUnavailable = errors.New("question manager unavailable")
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
		return nil, ErrQuestionManagerUnavailable
	}
	if id == "" {
		return nil, ErrInvalidQuestionID
	}

	manager.mutex.Lock()
	defer manager.mutex.Unlock()

	if _, ok := manager.subscribers[id]; ok {
		return nil, ErrQuestionAlreadySubscribed
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
