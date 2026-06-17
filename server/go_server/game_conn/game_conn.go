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

func ReadMessageAsAnswer(message string) (AnswerFromGameServer, bool, error) {
	var gameAnswer AnswerFromGameServer

	if err := json.Unmarshal([]byte(message), &gameAnswer); err != nil {
		return AnswerFromGameServer{}, false, err
	}

	if gameAnswer.Question == "" || gameAnswer.Id == "" {
		return AnswerFromGameServer{}, false, nil
	}

	return gameAnswer, true, nil
}

func ReadMessageAsEventBatchList(message string) ([]protocol.EventBatch, bool, error) {
	var gameEventBatches []protocol.EventBatch

	if err := json.Unmarshal([]byte(message), &gameEventBatches); err != nil {
		return nil, false, nil
	}

	if len(gameEventBatches) == 0 {
		return nil, false, nil
	}

	for _, gameEventBatch := range gameEventBatches {
		if !gameEventBatch.IsValid() {
			return nil, false, nil
		}
	}

	return gameEventBatches, true, nil
}

func ReadMessageAsEventBatch(message string) (protocol.EventBatch, bool, error) {
	var gameEventBatch protocol.EventBatch

	if err := json.Unmarshal([]byte(message), &gameEventBatch); err != nil {
		return protocol.EventBatch{}, false, nil
	}

	if !gameEventBatch.IsValid() {
		return protocol.EventBatch{}, false, nil
	}

	return gameEventBatch, true, nil
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
