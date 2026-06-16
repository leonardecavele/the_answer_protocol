package client_conn

import (
	"bufio"
	"encoding/json"
	"errors"
	"go_server/client_conn/tap_commands"
	"go_server/game_conn"
	"go_server/logger"
	"io"
	"strings"
)

func parseCommand(msg string) (string, string, string) {
	msg = strings.TrimRight(msg, "\r\n")

	if msg == "" {
		return "", "", responseEmptyCommand
	}

	command, args, _ := strings.Cut(msg, " ")
	if _, ok := tap_commands.tapCommands[command]; ok {
		return command, args, ""
	}

	return "", "", responseCommandNotFound
}

func handleTapCommand(str string, client *Client, gameServer *game_conn.GameServerManager) (string, error) {
	response := ""

	cmd, args, response := parseCommand(str)
	if response != "" {
		return response, nil
	}

	response, err := tap_commands.tapCommands[cmd](args, client, gameServer)
	if err != nil {
		return "", err
	}

	return response, nil
}

func formatClientEvent(event game_conn.EventFromGameServer) (string, error) {
	message := "EVT " + event.EventName
	if event.Data == nil {
		return message, nil
	}

	data, ok := event.Data.(string)
	if !ok {
		dataBytes, err := json.Marshal(event.Data)
		if err != nil {
			return "", err
		}
		data = string(dataBytes)
	}

	if data == "" {
		return message, nil
	}
	return message + " " + data, nil
}

func handleClientEvents(client *Client, done <-chan struct{}) {
	for {
		select {
		case <-done:
			return
		case event := <-client.eventChan:
			message, err := formatClientEvent(event)
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

func HandleClient(client *Client, gameServer *game_conn.GameServerManager) {
	defer func() {
		if err := client.DeleteClient(gameServer); err != nil {
			logger.AppLogger.Error("%s Erase client error: %v\n", client.Id, err)
		}
	}()

	stopListeningEvents := make(chan struct{})
	defer close(stopListeningEvents)

	logger.AppLogger.Info("%s Connected", client.Id)
	defer logger.AppLogger.Info("%s Disconnected", client.Id)

	if err := client.Write(responseHello); err != nil {
		logger.AppLogger.Error("%s Client Write Error: %v\n", client.Id, err)
		return
	}
	logger.AppLogger.Info("%s Client Write: %s", client.Id, responseHello)

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
		response, err := handleTapCommand(str, client, gameServer)
		if err != nil {
			logger.AppLogger.Error("%s Command error: %v\n", client.Id, err)
			response = responseGameServerClosed
		}
		if response == "" {
			continue
		}
		if err := client.Write(response); err != nil {
			logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
			return
		}
		if response == responseBye {
			return
		}
		logger.AppLogger.Info("%s Client Write: %s", client.Id, response)
	}
}
