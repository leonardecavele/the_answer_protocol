package client_conn

import (
	"go_server/rust_conn"
	"strings"
)

type handleTapCommandArgs func(args string, client *Client, rustServer *rust_conn.RustServer) (string, error)

var tapCommands = map[string]handleTapCommandArgs{
	// CORE
	"CONNECT": handleConnectCommand,
	"LOOK":    handleLookCommand,
	"MOVE":    handleMoveCommand,
	"QUIT":    handleQuitCommand,
	// COMMUNICATION
	"CHAT": handleChatCommand,
	"WHO":  handleWhoCommand,
	// GROUP
	"GROUP CREATE": handleGroupCreateCommand,
	"GROUP INVITE": handleGroupInviteCommand,
	"GROUP JOIN":   handleGroupJoinCommand,
	"GROUP LEAVE":  handleGroupLeaveCommand,
	// RESOURCE INTERACTION
	"TAKE":      handleTakeCommand,
	"DROP":      handleDropCommand,
	"INVENTORY": handleInventoryCommand,
	"TALK":      handleTalkCommand,
	"ATTACK":    handleAttackCommand,
	"STATUS":    handleStatusCommand,
	"QUEST":     handleQuestCommand,
	"QUESTS":    handleQuestsCommand,
}

func RustTryWriteCommand(command rust_conn.CommandToRust, rustServer *rust_conn.RustServer) error {
	if rustServer == nil {
		return nil
	}
	return rustServer.WriteCommand(command)
}

// CORE

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

	if err := RustTryWriteCommand(command, rustServer); err != nil {
		return "", err
	}

	return responseConnected, nil
}

func handleLookCommand(args string, client *Client, rustServer *rust_conn.RustServer) (string, error) {
	if rustServer == nil {
		return responseRustServerShutdown, nil
	}

	if client.State != AUTHENTICATED {
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

func handleMoveCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleQuitCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	if args != "" {
		return responseInvalidArguments, nil
	}

	return responseBye, nil
}

// COMMUNICATION

func handleChatCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleWhoCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

// GROUP

// issue because of SPACE
func handleGroupCreateCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleGroupInviteCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleGroupJoinCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleGroupLeaveCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

// RESOURCE INTERACTION

func handleTakeCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleDropCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleInventoryCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleTalkCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleAttackCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleStatusCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleQuestCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}

func handleQuestsCommand(args string, client *Client, _ *rust_conn.RustServer) (string, error) {
	return "", nil
}
