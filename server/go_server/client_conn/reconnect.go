package client_conn

import (
	"go_server/logger"
	"go_server/rust_conn"
)

func ReconnectPlayersToRust(rustServer *rust_conn.RustServerManager) error {
	for _, username := range ConnectedUsernames() {
		command := rust_conn.CommandToRust{
			Player:    username,
			Command:   "CONNECT",
			Arguments: username,
		}

		if err := rustServer.WriteCommand(command); err != nil {
			return err
		}
		logger.AppLogger.Info("Reconnected %s to Rust server", username)
	}

	return nil
}
