package client_conn

import (
	"errors"
	"strings"
)

type handleTapCommandArgs func(args string, client Client) (string, error)

var tapCommands = map[string]handleTapCommandArgs{
	"CONNECT": handleConnectCommand,
}

func handleConnectCommand(args string, client Client) (string, error) {
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

	if strings.Contains(args, " ") || !isValidUsername(args) {
		// TODO (edit response according to protocol)
		return "ERR 6060 INVALID USERNAME PLACEHOLDER\n", errors.New("invalid username")
	}

	response, err := client.SetUsername(args)
	if err != nil {
		// TODO (edit response according to protocol)
		return response, errors.New("username taken")
	}

	return "OK connected\n", nil
}
