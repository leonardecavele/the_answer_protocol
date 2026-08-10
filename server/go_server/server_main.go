package main

import (
	"bufio"
	"fmt"
	"go_server/client_conn"
	"go_server/config"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/helper"
	"go_server/logger"
	"go_server/session"
	"net"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"sync"
	"syscall"
)

func shutdownServer(quit chan struct{}, listener net.Listener, stopOnce *sync.Once) {
	stopOnce.Do(func() {
		close(quit)
		logger.AppLogger.Info("Server stopped.")
		if listener != nil {
			listener.Close()
		}
	})
}

func main() {
	quit := make(chan struct{})
	var stopOnce sync.Once
	var listener net.Listener

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	defer signal.Stop(signals)

	go func() {
		<-signals
		shutdownServer(quit, listener, &stopOnce)
	}()

	go func() {
		scanner := bufio.NewScanner(os.Stdin)

		for scanner.Scan() {
			input := strings.ToLower(strings.TrimSpace(scanner.Text()))

			if _, ok := config.QuitCommands[input]; ok {
				shutdownServer(quit, listener, &stopOnce)
				return
			}
		}
	}()

	gameServerManager := &game_conn.GameServerManager{}
	room := session.NewRoom()

	go gameServerManager.HandleGameServer(
		quit,
		room.ReconnectPlayersToGameServer,
		room.RouteCommand,
		room.BroadcastEvent,
	)

	listener, listenErr := net.Listen("tcp", ":"+strconv.Itoa(config.GoServerPort))
	if listenErr != nil {
		logger.AppLogger.Error(fmt.Sprint(listenErr))
		os.Exit(int(serverError.CodeListenerError))
	}
	defer listener.Close()

	logger.AppLogger.Info("TCP server started on " + helper.GetServerIP() + ":" + strconv.Itoa(config.GoServerPort))

	for {
		conn, err := listener.Accept()
		if err != nil {
			select {
			case <-quit:
				return
			default:
				logger.AppLogger.Error("Accept error:", err)
				continue
			}
		}

		go client_conn.HandleClient(session.NewClient(conn, room), gameServerManager)
	}
}
