package game_conn

import (
	"bufio"
	"encoding/json"
	"errors"
	"go_server/config"
	serverError "go_server/error"
	"go_server/logger"
	"go_server/protocol"
	"io"
	"net"
	"strings"
	"sync"
)

type GameServer struct {
	Conn       net.Conn
	Writer     *bufio.Writer
	PrintMutex sync.Mutex
}

func (gameServer *GameServer) currentConn() net.Conn {
	gameServer.PrintMutex.Lock()
	defer gameServer.PrintMutex.Unlock()

	return gameServer.Conn
}

func (gameServer *GameServer) Close() error {
	gameServer.PrintMutex.Lock()
	conn := gameServer.Conn
	gameServer.Conn = nil
	gameServer.Writer = nil
	gameServer.PrintMutex.Unlock()

	if conn == nil {
		return nil
	}
	return conn.Close()
}

func (gameServer *GameServer) Write(message string) error {
	gameServer.PrintMutex.Lock()
	defer gameServer.PrintMutex.Unlock()

	if gameServer.Writer == nil {
		return serverError.ErrGameServerNotConnected
	}

	if _, err := gameServer.Writer.WriteString(message); err != nil {
		return err
	}
	if err := gameServer.Writer.WriteByte('\n'); err != nil {
		return err
	}
	if err := gameServer.Writer.Flush(); err != nil {
		return err
	}

	logger.AppLogger.Info("Game server write: %s", message)
	return nil
}

func (gameServer *GameServer) WriteCommand(command any) error {
	message, err := json.Marshal(command)
	if err != nil {
		return err
	}

	err = gameServer.Write(string(message))
	return err
}

func (gameServer *GameServer) Read(
	quit <-chan struct{},
	routeCommand func(username string, command CommandFromGameServer) bool,
	routeEvent func(username string, event protocol.Event) bool,
) {
	conn := gameServer.currentConn()
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
				logger.AppLogger.Error("Game server read error: %v", err)
			}
			logger.AppLogger.Info("Game server disconnected")
			return
		}

		message = strings.TrimRight(message, "\r\n")
		logger.AppLogger.Info("Game server read: %s", message)

		if message == config.GameConfirmationMessage {
			continue
		}

		gameEvents, ok, err := ReadMessageAsEvents(message)
		if err != nil {
			logger.AppLogger.Error("Game server invalid message: %v", err)
			continue
		}
		if ok && routeEvent != nil {
			for _, gameEvent := range gameEvents {
				routeEvent(gameEvent.Player, gameEvent)
			}
			continue
		}

		gameEvent, ok, err := ReadMessageAsEvent(message)
		if err != nil {
			logger.AppLogger.Error("Game server invalid message: %v", err)
			continue
		}
		if ok && routeEvent != nil {
			routeEvent(gameEvent.Player, gameEvent)
			continue
		}

		gameCommand, ok, err := ReadMessageAsCommand(message)
		if err != nil {
			logger.AppLogger.Error("Game server invalid message: %v", err)
			continue
		}
		if ok && routeCommand != nil {
			routeCommand(gameCommand.Player, gameCommand)
		}
	}
}
