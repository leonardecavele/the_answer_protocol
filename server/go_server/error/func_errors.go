package error

import (
	"errors"
)

var (
	ErrConnectionManagerMissing    = errors.New("connection manager unavailable")
	ErrInvalidConnection           = errors.New("invalid connection")
	ErrConnectionAlreadySubscribed = errors.New("connection already subscribed")
	ErrMaxConnection               = errors.New("maximum number of connections reached")
	ErrRateLimitExceeded           = errors.New("rate limit exceeded")
	ErrReadStringTooLong           = errors.New("read string exceeds maximum size")
	ErrGameServerNotConnected      = errors.New("game server not connected")
	ErrGameServerAnswerTimeout     = errors.New("timeout waiting for game server answer")
	ErrInvalidQuestionID           = errors.New("invalid question id")
	ErrQuestionSubscribed          = errors.New("question already subscribed")
	ErrQuestionManagerMissing      = errors.New("question manager unavailable")
)
