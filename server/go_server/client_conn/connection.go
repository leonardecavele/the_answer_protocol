package client_conn

import (
	"bufio"
	"errors"
	"io"
	"strconv"
	"strings"
)

import (
	"go_server/config"
	"go_server/logger"
)

func parseCommand(msg string) (string, string, error) {
	msg = strings.TrimRight(msg, "\r\n")

	if msg == "" {
		return "", "", errors.New("empty command")
	}

	command, args, found := strings.Cut(msg, " ")
	if !found {
		return "", "", errors.New("invalid command")
	}
	if _, ok := tapCommands[command]; ok {
		return command, args, nil
	}

	return "", "", errors.New("unknown command")
}

func handleTapCommand(str string, client Client) string {
	response := ""

	cmd, args, err := parseCommand(str)
	if err != nil {
		// TODO (edit response according to protocol)
		response = "ERR 400 COMMAND_NOT_FOUND\n"
		logger.AppLogger.Info("%s Invalid command: %v\n", client.Id, err)
		return response
	}

	response, err = tapCommands[cmd](args, client)
	if err != nil {
		logger.AppLogger.Info("%s Invalid command: %v\n", client.Id, err)
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

func HandleClient(client Client) { //, rustServer *rust_conn.RustServer) {
	defer client.EraseClient()

	logger.AppLogger.Info("%s Connected", client.Id)
	defer logger.AppLogger.Info("%s Disconnected", client.Id)

	if _, err := client.Conn.Write([]byte("OK hello proto=" + strconv.Itoa(config.ProtocolVersion) + "\n")); err != nil {
		logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
		return
	}

	reader := bufio.NewReader(client.Conn)
	for {
		str, err := reader.ReadString('\n')
		if err != nil {
			if !errors.Is(err, io.EOF) {
				logger.AppLogger.Error("%s Read error: %v\n", client.Id, err)
			}
			return
		}

		logger.AppLogger.Info("%s Sent: %s", client.Id, str)
		response := handleTapCommand(str, client)
		if _, err := client.Conn.Write([]byte(response)); err != nil {
			logger.AppLogger.Error("%s Write error: %v\n", client.Id, err)
			return
		}
	}
}
