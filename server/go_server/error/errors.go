package error

import (
	"errors"
)

var (
	ErrGameServerNotConnected = errors.New("game server not connected")
	ErrInvalidQuestionID      = errors.New("invalid question id")
	ErrQuestionSubscribed     = errors.New("question already subscribed")
	ErrQuestionManagerMissing = errors.New("question manager unavailable")
)
