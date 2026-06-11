package rust_conn

import (
	"bufio"
	"encoding/json"
	"errors"
	"io"
	"net"
	"strings"
	"time"
)

import (
	"go_server/config"
	"go_server/logger"
)

func dialRust(addr string, quit <-chan struct{}) net.Conn {
	for {
		select {
		case <-quit:
			return nil
		default:
		}

		conn, err := net.Dial("tcp", addr)
		if err == nil {
			logger.AppLogger.Info("Connected to Rust server")
			return conn
		}

		logger.AppLogger.Info("Rust server unavailable at %s, retrying in %d seconds", addr, config.RustConnectionRetryDelay)
		select {
		case <-quit:
			return nil
		case <-time.After(time.Second * config.RustConnectionRetryDelay):
		}
	}
}

func ConnectToRust(addr string, quit <-chan struct{}) *RustServer {
	conn := dialRust(addr, quit)
	if conn == nil {
		return nil
	}

	return &RustServer{
		Conn:   conn,
		Writer: bufio.NewWriter(conn),
	}
}

func (rustServer *RustServer) currentConn() net.Conn {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

	return rustServer.Conn
}

func (rustServer *RustServer) Close() error {
	rustServer.PrintMutex.Lock()
	conn := rustServer.Conn
	rustServer.Conn = nil
	rustServer.Writer = nil
	rustServer.PrintMutex.Unlock()

	if conn == nil {
		return nil
	}
	return conn.Close()
}

func (rustServer *RustServer) Write(message string) error {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

	if rustServer.Writer == nil {
		return errors.New("rust server not connected")
	}

	if _, err := rustServer.Writer.WriteString(message); err != nil {
		return err
	}
	if err := rustServer.Writer.WriteByte('\n'); err != nil {
		return err
	}
	if err := rustServer.Writer.Flush(); err != nil {
		return err
	}

	logger.AppLogger.Info("Rust Write: %s", message)
	return nil
}

func (rustServer *RustServer) WriteCommand(command any) error {
	message, err := json.Marshal(command)
	if err != nil {
		return err
	}

	err = rustServer.Write(string(message))
	return err
}

func ReadMessageAsEvents(message string) ([]EventFromRust, bool, error) {
	var rustEvents []EventFromRust

	if err := json.Unmarshal([]byte(message), &rustEvents); err != nil {
		return nil, false, nil
	}

	if len(rustEvents) == 0 {
		return nil, false, nil
	}

	for _, rustEvent := range rustEvents {
		if rustEvent.Player == "" || rustEvent.EventName == "" {
			return nil, false, nil
		}
	}

	return rustEvents, true, nil
}

func ReadMessageAsEvent(message string) (EventFromRust, bool, error) {
	var rustEvent EventFromRust

	if err := json.Unmarshal([]byte(message), &rustEvent); err != nil {
		return EventFromRust{}, false, err
	}

	if rustEvent.Player == "" || rustEvent.EventName == "" {
		return EventFromRust{}, false, nil
	}

	return rustEvent, true, nil
}

func ReadMessageAsCommand(message string) (CommandFromRust, bool, error) {
	var rustCommand CommandFromRust

	if err := json.Unmarshal([]byte(message), &rustCommand); err != nil {
		return CommandFromRust{}, false, err
	}

	if rustCommand.Player == "" || rustCommand.Command == "" {
		return CommandFromRust{}, false, nil
	}

	return rustCommand, true, nil
}

func (rustServer *RustServer) Read(
	quit <-chan struct{},
	routeCommand func(username string, command string) bool,
	routeEvent func(username string, event string) bool,
) {
	conn := rustServer.currentConn()
	if conn == nil {
		return
	}

	reader := bufio.NewReader(conn)
	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			select {
			case <-quit:
				return
			default:
			}
			if !errors.Is(err, io.EOF) {
				logger.AppLogger.Error("Rust read error: %v", err)
			}
			logger.AppLogger.Info("Rust server disconnected")
			return
		}

		message = strings.TrimRight(message, "\r\n")
		logger.AppLogger.Info("Rust Read: %s", message)

		if message == config.RustConfirmationMessage {
			continue
		}

		rustEvents, ok, err := ReadMessageAsEvents(message)
		if err != nil {
			logger.AppLogger.Error("Rust invalid message: %v", err)
			continue
		}
		if ok && routeEvent != nil {
			for _, rustEvent := range rustEvents {
				eventMessage, err := json.Marshal(rustEvent)
				if err != nil {
					logger.AppLogger.Error("Rust invalid event: %v", err)
					continue
				}
				routeEvent(rustEvent.Player, string(eventMessage))
			}
			continue
		}

		rustEvent, ok, err := ReadMessageAsEvent(message)
		if err != nil {
			logger.AppLogger.Error("Rust invalid message: %v", err)
			continue
		}
		if ok && routeEvent != nil {
			routeEvent(rustEvent.Player, message)
			continue
		}

		rustCommand, ok, err := ReadMessageAsCommand(message)
		if err != nil {
			logger.AppLogger.Error("Rust invalid message: %v", err)
			continue
		}
		if ok && routeCommand != nil {
			routeCommand(rustCommand.Player, message)
		}
	}
}
