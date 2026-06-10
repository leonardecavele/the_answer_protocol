package client_conn

import (
	"bufio"
	"errors"
	"go_server/rust_conn"
	"io"
	"strings"
)

import (
	"go_server/logger"
)

func parseCommand(msg string) (string, string, error) {
	msg = strings.TrimRight(msg, "\r\n")

	if msg == "" {
		return "", "", errEmptyCommand
	}

	command, args, _ := strings.Cut(msg, " ")
	if _, ok := tapCommands[command]; ok {
		return command, args, nil
	}

	return "", "", errUnknownCommand
}

func handleTapCommand(str string, client *Client, rustServer *rust_conn.RustServer) string {
	response := ""

	cmd, args, err := parseCommand(str)
	if err != nil {
		response = responseCommandNotFound
		return response
	}

	response, err = tapCommands[cmd](args, client, rustServer)
	if err != nil {
		return response
	}

	return response
}

func handleClientEvents(client *Client, done <-chan struct{}) {
	for {
		select {
		case <-done:
			return
		case event := <-client.eventChan:
			response := event + "\n"
			if err := client.Write("EVT " + string(response)); err != nil {
				logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
				return
			}
			logger.AppLogger.Info("%s Client Write: %s", client.Id, response)
		}
	}
}

func HandleClient(client *Client, rustServer *rust_conn.RustServer) {
	defer client.EraseClient()

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
		response := handleTapCommand(str, client, rustServer)
		if response == "" {
			continue
		}
		if err := client.Write(response); err != nil {
			logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
			return
		}
		logger.AppLogger.Info("%s Client Write: %s", client.Id, response)
	}
}
