package client_conn

import (
	"bufio"
	"errors"
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

	command, args, found := strings.Cut(msg, " ")
	if !found {
		return "", "", errInvalidCommand
	}
	if _, ok := tapCommands[command]; ok {
		return command, args, nil
	}

	return "", "", errUnknownCommand
}

func handleTapCommand(str string, client *Client) string {
	response := ""

	cmd, args, err := parseCommand(str)
	if err != nil {
		response = responseCommandNotFound
		return response
	}

	response, err = tapCommands[cmd](args, client)
	if err != nil {
		return response
	}

	return response

	//fmt.Printf("[%v] Received PING from client %d\n", time.Now().Format(config.LogFormat), i)
	//fmt.Printf("[%v] Sending PING to Rust\n", time.Now().Format(config.LogFormat))
	//err = rustServer.Write("PING")
	//if err != nil {
	//	fmt.Println("Rust send error:", err)
	//	return
	//}
}

func HandleClient(client *Client) { //, rustServer *rust_conn.RustServer) {
	defer client.EraseClient()

	logger.AppLogger.Info("%s Connected", client.Id)
	defer logger.AppLogger.Info("%s Disconnected", client.Id)

	if _, err := client.Conn.Write([]byte(responseHello)); err != nil {
		logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
		return
	}
	logger.AppLogger.Info("%s Write: %s", client.Id, responseHello)

	reader := bufio.NewReader(client.Conn)
	for {
		str, err := reader.ReadString('\n')
		if err != nil {
			if !errors.Is(err, io.EOF) {
				logger.AppLogger.Error("%s Read error: %v\n", client.Id, err)
			}
			return
		}

		logger.AppLogger.Info("%s Read: %s", client.Id, str)
		response := handleTapCommand(str, client)
		if _, err := client.Conn.Write([]byte(response)); err != nil {
			logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
			return
		}
		logger.AppLogger.Info("%s Write: %s", client.Id, response)
	}
}
