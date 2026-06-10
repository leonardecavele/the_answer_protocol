package client_conn

import (
	"go_server/rust_conn"
	"strings"
)

type handleTapCommandArgs func(args string, client *Client, rustServer *rust_conn.RustServer) (string, error)

var tapCommands = map[string]handleTapCommandArgs{
	"CONNECT": handleConnectCommand,
	"LOOK":    handleLookCommand,
	"QUIT":    handleQuitCommand,
}

func handleConnectCommand(args string, client *Client, rustServer *rust_conn.RustServer) (string, error) {
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
		return responseInvalidUsername, nil
	}

	if response := client.SetUsername(strings.ToUpper(args)); response != "" {
		return response, nil
	}

	command := rust_conn.CommandToRust{
		Player:    client.Username,
		Command:   "CONNECT",
		Arguments: args,
	}

	if err := rustServer.WriteCommand(command); err != nil {
		return "", err
	}

	return responseConnected, nil
}

func handleLookCommand(args string, client *Client, rustServer *rust_conn.RustServer) (string, error) {
	if client.Username == "" {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
	}

	command := rust_conn.CommandToRust{
		Player:    client.Username,
		Command:   "LOOK",
		Arguments: args,
	}

	if err := rustServer.WriteCommand(command); err != nil {
		return "", err
	}

	return "OK " + client.ReadCommand(), nil
}

func handleQuitCommand(args string, client *Client, rustServer *rust_conn.RustServer) (string, error) {
	if client.Username == "" {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
	}

	command := rust_conn.CommandToRust{
		Player:    client.Username,
		Command:   "QUIT",
		Arguments: args,
	}

	if err := rustServer.WriteCommand(command); err != nil {
		return "", err
	}

	return responseBye, nil
}
