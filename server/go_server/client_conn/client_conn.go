package client_conn

import (
	"bufio"
	"errors"
	"go_server/client_conn/tap_commands"
	"go_server/game_conn"
	"go_server/logger"
	"go_server/protocol"
	"go_server/session"
	"io"
	"strings"
)

func parseCommand(msg string) (string, string, string) {
	msg = strings.TrimRight(msg, "\r\n")

	if msg == "" {
		return "", "", protocol.ResponseEmptyCommand
	}

	command, args, _ := strings.Cut(msg, " ")
	if _, ok := tap_commands.TapCommands[command]; ok {
		return command, args, ""
	}

	return "", "", protocol.ResponseCommandNotFound
}

func handleTapCommand(str string, client *session.Client, gameServerManager *game_conn.GameServerManager) (string, error) {
	response := ""

	cmd, args, response := parseCommand(str)
	if response != "" {
		return response, nil
	}

	response, err := tap_commands.TapCommands[cmd](args, client, gameServerManager)
	if err != nil {
		return "", err
	}

	return response, nil
}

func handleClientEvents(client *session.Client, done <-chan struct{}) {
	for {
		select {
		case <-done:
			return
		case event := <-client.Events():
			message, err := protocol.FormatEvent(event)
			if err != nil {
				logger.AppLogger.Error("%s Invalid event: %v\n", client.Id, err)
				return
			}
			if err := client.Write(message); err != nil {
				logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
				return
			}
			logger.AppLogger.Info("%s Client Write: %s\n", client.Id, message)
		}
	}
}

func HandleClient(client *session.Client, gameServerManager *game_conn.GameServerManager) {
	defer func() {
		if err := client.DeleteClient(gameServerManager); err != nil {
			logger.AppLogger.Error("%s Erase client error: %v\n", client.Id, err)
		}
	}()

	stopListeningEvents := make(chan struct{})
	defer close(stopListeningEvents)

	logger.AppLogger.Info("%s Connected", client.Id)
	defer logger.AppLogger.Info("%s Disconnected", client.Id)

	if err := client.Write(protocol.ResponseHello); err != nil {
		logger.AppLogger.Error("%s Client Write Error: %v\n", client.Id, err)
		return
	}
	logger.AppLogger.Info("%s Client Write: %s", client.Id, protocol.ResponseHello)

	go handleClientEvents(client, stopListeningEvents)

	reader := bufio.NewReader(client.Conn)
	for {
		str, err := reader.ReadString('\n')
		if err != nil {
			if !errors.Is(err, io.EOF) {
				logger.AppLogger.Error("%s Read error: %v\n", client.Id, err)
			}
			return
		}

		logger.AppLogger.Info("%s Client Read: %s", client.Id, str)
		response, err := handleTapCommand(str, client, gameServerManager)
		if err != nil {
			logger.AppLogger.Error("%s Command error: %v\n", client.Id, err)
			response = protocol.ResponseGameServerClosed
		}
		if response == "" {
			continue
		}
		if err := client.Write(response); err != nil {
			logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
			return
		}
		if response == protocol.ResponseBye {
			return
		}
		logger.AppLogger.Info("%s Client Write: %s", client.Id, response)
	}
}
