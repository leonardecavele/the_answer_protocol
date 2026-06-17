package game_conn

import (
	"bufio"
	"encoding/json"
	"go_server/config"
	"go_server/logger"
	"go_server/protocol"
	"net"
	"time"
)

func isValidEvent(event protocol.Event) bool {
	return event.Player != "" && event.EventName != ""
}

func dialGameServer(addr string, quit <-chan struct{}) net.Conn {
	for {
		select {
		case <-quit:
			return nil
		default:
		}

		conn, err := net.Dial("tcp", addr)
		if err == nil {
			logger.AppLogger.Info("Connected to Game server")
			return conn
		}

		logger.AppLogger.Info("Game server unavailable at %s, retrying in %d seconds", addr, config.GameConnectionRetryDelay)
		select {
		case <-quit:
			return nil
		case <-time.After(time.Second * config.GameConnectionRetryDelay):
		}
	}
}

func ConnectToGameServer(addr string, quit <-chan struct{}) *GameServer {
	conn := dialGameServer(addr, quit)
	if conn == nil {
		return nil
	}

	return &GameServer{
		Conn:   conn,
		Writer: bufio.NewWriter(conn),
	}
}

func ReadMessageAsEvents(message string) ([]protocol.Event, bool, error) {
	var gameEvents []protocol.Event

	if err := json.Unmarshal([]byte(message), &gameEvents); err != nil {
		var gameEvent protocol.Event
		if err := json.Unmarshal([]byte(message), &gameEvent); err != nil {
			return nil, false, nil
		}
		gameEvents = []protocol.Event{gameEvent}
	}

	if len(gameEvents) == 0 {
		return nil, false, nil
	}

	for _, gameEvent := range gameEvents {
		if !isValidEvent(gameEvent) {
			return nil, false, nil
		}
	}

	return gameEvents, true, nil
}

func ReadMessageAsEvent(message string) (protocol.Event, bool, error) {
	var gameEvent protocol.Event

	if err := json.Unmarshal([]byte(message), &gameEvent); err != nil {
		return protocol.Event{}, false, err
	}

	if gameEvent.Player == "" || gameEvent.EventName == "" {
		return protocol.Event{}, false, nil
	}

	return gameEvent, true, nil
}

func ReadMessageAsCommand(message string) (CommandFromGameServer, bool, error) {
	var gameCommand CommandFromGameServer

	if err := json.Unmarshal([]byte(message), &gameCommand); err != nil {
		return CommandFromGameServer{}, false, err
	}

	if gameCommand.Player == "" || gameCommand.Command == "" {
		return CommandFromGameServer{}, false, nil
	}

	return gameCommand, true, nil
}
