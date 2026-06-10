package client_conn

import "errors"

var (
	// Command
	errEmptyCommand   = errors.New("empty command")
	errInvalidCommand = errors.New("invalid command")
	errUnknownCommand = errors.New("unknown command")

	// Client username
	errClientAlreadyHasUsername = errors.New("client already has username")
	errUsernameAlreadyUsed      = errors.New("username already used")
	errInvalidUsername          = errors.New("invalid username")

	// Room
	errRoomFull = errors.New("room is full")
)
