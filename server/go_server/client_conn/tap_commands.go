package client_conn

import (
	"errors"
	"fmt"
	"go_server/game_conn"
	"strings"
)

type handleTapCommandArgs func(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error)

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

func handleGameCommandError(response game_conn.CommandFromGameServer) string {
	switch response.ErrorCode {
	case 201:
		return responseUsernameAlreadyUsed
	case 301:
		return responseNoExit
	case 400:
		return responseInvalidArguments
	case 401:
		return responseNotInGroup
	case 402:
		return responseAlreadyInGroup
	case 405:
		return responseNpcNotHostile
	case 406:
		return responseNoQuestAvailable
	case 900:
		return responseConnectionFailed
	case 901:
		return responseSendFailed
	default:
		return fmt.Sprintf("ERR %03d UNKNOWN_ERROR", response.ErrorCode)
	}
}

// CORE

func handleConnectCommand(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
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

	command := game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "CONNECT",
		Arguments: args,
	}

	if err := gameServer.WriteCommand(command); err != nil && !errors.Is(err, game_conn.ErrGameServerNotConnected) {
		return "", err
	}

	return responseConnected, nil
}

func handleLookCommand(args string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
	if !gameServer.IsConnected() {
		return responseGameServerClosed, nil
	}

	if client.State != AUTHENTICATED {
		return responseNotConnected, nil
	}
	if args != "" {
		return responseInvalidArguments, nil
	}

	command := game_conn.CommandToGameServer{
		Player:    client.Username,
		Command:   "LOOK",
		Arguments: args,
	}

	if err := gameServer.WriteCommand(command); err != nil {
		return "", err
	}

	response := client.ReadCommand()
	if errorResponse := handleGameCommandError(response); errorResponse != "" {
		return errorResponse, nil
	}

	return "OK " + response.Data, nil
}

func handleMoveCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuitCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	if args != "" {
		return responseInvalidArguments, nil
	}

	return responseBye, nil
}

// COMMUNICATION

func handleChatCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleWhoCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

// GROUP

// issue because of SPACE
func handleGroupCreateCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleGroupInviteCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleGroupJoinCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleGroupLeaveCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

// RESOURCE INTERACTION

func handleTakeCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleDropCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleInventoryCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleTalkCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleAttackCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleStatusCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestsCommand(args string, client *Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
