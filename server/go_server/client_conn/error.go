package client_conn

import "errors"

var (
	// Network
	errNotConnected = errors.New("not connected")

	// Command
	errEmptyCommand     = errors.New("empty command")
	errUnknownCommand   = errors.New("unknown command")
	errInvalidArguments = errors.New("invalid arguments")

	// Client username
	errClientAlreadyHasUsername = errors.New("client already has username")
	errUsernameAlreadyUsed      = errors.New("username already used")
	errInvalidUsername          = errors.New("invalid username")

	// Room
	errRoomFull = errors.New("room is full")
)
