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

func ConnectToRust(addr string, quit <-chan struct{}) *RustServer {
	for {
		select {
		case <-quit:
			return nil
		default:
		}

		conn, err := net.Dial("tcp", addr)
		if err == nil {
			logger.AppLogger.Info("Connected to Rust server")
			return &RustServer{
				Conn:   conn,
				Writer: bufio.NewWriter(conn),
			}
		}

		logger.AppLogger.Info("Rust server unavailable at %s, retrying in %d seconds", addr, config.RustConnectionRetryDelay)
		select {
		case <-quit:
			return nil
		case <-time.After(time.Second * config.RustConnectionRetryDelay):
		}
	}
}

func (rustServer *RustServer) Write(message string) error {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

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
	onClose func(),
	routeCommand func(username string, command string) bool,
	routeEvent func(username string, event string) bool,
) {
	reader := bufio.NewReader(rustServer.Conn)

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
			if onClose != nil {
				onClose()
			}
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
