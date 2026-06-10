package client_conn

import (
	"errors"
	"strings"
)

type handleTapCommandArgs func(args string, client *Client) (string, error)

var tapCommands = map[string]handleTapCommandArgs{
	"CONNECT": handleConnectCommand,
}

func handleConnectCommand(args string, client *Client) (string, error) {
	isValidUsername := func(username string) bool {
		if username == "" {
			return false
		}
		for _, c := range username {
			if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
				(c >= '0' && c <= '9') || c == '-' || c == '_' {
				continue
			}
			return false
		}
		return true
	}

	if !isValidUsername(args) {
		return responseInvalidUsername, errInvalidUsername
	}

	err := client.SetUsername(strings.ToUpper(args))
	if err != nil {
		switch {
		case errors.Is(err, errClientAlreadyHasUsername):
			return responseAlreadyConnected, err
		case errors.Is(err, errRoomFull):
			return responseRoomFull, err
		case errors.Is(err, errUsernameAlreadyUsed):
			return responseUsernameAlreadyUsed, err
		}
		return "", err
	}

	return responseConnected, nil
}
