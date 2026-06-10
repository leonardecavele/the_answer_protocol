package rust_conn

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"strings"
	"time"
)

import (
	"go_server/config"
	"go_server/logger"
)

func ConnectToRust(addr string) *RustServer {
	for {
		conn, err := net.Dial("tcp", addr)
		if err == nil {
			logger.AppLogger.Info("Connected to Rust server")
			return &RustServer{Conn: conn}
		}

		logger.AppLogger.Info("Rust server unavailable at %s, retrying in %d seconds", addr, config.RustConnectionRetryDelay)
		time.Sleep(time.Second * config.RustConnectionRetryDelay)
	}
}

func (rustServer *RustServer) Write(message string) error {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

	_, err := fmt.Fprintf(rustServer.Conn, "%s\n", message)
	return err
}

func (rustServer *RustServer) WriteCommand(command any) error {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

	message, err := json.Marshal(command)
	if err != nil {
		return err
	}

	_, err = fmt.Fprintf(rustServer.Conn, "%s\n", message)
	if err == nil {
		logger.AppLogger.Info("Rust Write: %s", message)
	}
	return err
}

func (rustServer *RustServer) Read(
	onClose func(),
	routeCommand func(username string, command string) bool,
	routeEvent func(username string, event string) bool,
) {
	reader := bufio.NewReader(rustServer.Conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			if !errors.Is(err, io.EOF) {
				logger.AppLogger.Error("Rust read error: %v", err)
			}
			if onClose != nil {
				onClose()
			}
			return
		}

		message = strings.TrimRight(message, "\r\n")
		logger.AppLogger.Info("Rust Read: %s", message)

		var rustEvent EventFromRust
		if err := json.Unmarshal([]byte(message), &rustEvent); err != nil {
			logger.AppLogger.Error("Rust invalid message: %v", err)
			continue
		}

		if rustEvent.Player != "" && rustEvent.EventName != "" && routeEvent != nil {
			routeEvent(rustEvent.Player, message)
			continue
		}

		var rustCommand CommandFromRust
		if err := json.Unmarshal([]byte(message), &rustCommand); err != nil {
			logger.AppLogger.Error("Rust invalid message: %v", err)
			continue
		}

		if rustCommand.Player != "" && rustCommand.Command != "" && routeCommand != nil {
			routeCommand(rustCommand.Player, message)
		}
	}
}
