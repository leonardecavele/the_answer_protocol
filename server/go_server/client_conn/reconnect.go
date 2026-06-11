package client_conn

import (
	"go_server/game_conn"
	"go_server/logger"
)

func ReconnectPlayersToGameServer(gameServer *game_conn.GameServerManager) error {
	for _, username := range ConnectedUsernames() {
		command := game_conn.CommandToGameServer{
			Player:    username,
			Command:   "CONNECT",
			Arguments: username,
		}

		if err := gameServer.WriteCommand(command); err != nil {
			return err
		}
		logger.AppLogger.Info("Reconnected %s to Game server", username)
	}

	return nil
}
