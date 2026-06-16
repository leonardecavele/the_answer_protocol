package tap_commands

import (
	"go_server/game_conn"
	"go_server/session"
)

func handleTakeCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleDropCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleInventoryCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleTalkCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleAttackCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleStatusCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}

func handleQuestCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	// grouped
	return "", nil
}

func handleQuestsCommand(args string, client *session.Client, _ *game_conn.GameServerManager) (string, error) {
	return "", nil
}
