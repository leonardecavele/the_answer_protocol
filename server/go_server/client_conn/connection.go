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

func parseCommand(msg string) (string, string, string) {
	msg = strings.TrimRight(msg, "\r\n")

	if msg == "" {
		return "", "", responseEmptyCommand
	}

	command, args, _ := strings.Cut(msg, " ")
	if _, ok := tapCommands[command]; ok {
		return command, args, ""
	}

	return "", "", responseCommandNotFound
}

func handleTapCommand(str string, client *Client, rustServer *rust_conn.RustServer) (string, error) {
	if rustServer == nil {
		return responseRustServerShutdown, nil
	}

	response := ""

	cmd, args, response := parseCommand(str)
	if response != "" {
		return response, nil
	}

	response, err := tapCommands[cmd](args, client, rustServer)
	if err != nil {
		return "", err
	}

	return response, nil
}

func handleClientEvents(client *Client, done <-chan struct{}) {
	for {
		select {
		case <-done:
			return
		case event := <-client.eventChan:
			if err := client.Write("EVT " + string(event)); err != nil {
				logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
				return
			}
			logger.AppLogger.Info("%s Client Write: %s\n", client.Id, event)
		}
	}
}

func HandleClient(client *Client, rustServer *rust_conn.RustServer, _ func()) {
	defer func() {
		if err := client.EraseClient(rustServer); err != nil {
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
		response, err := handleTapCommand(str, client, rustServer)
		if err != nil {
			logger.AppLogger.Error("%s Command error: %v\n", client.Id, err)
			response = responseRustServerShutdown
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
