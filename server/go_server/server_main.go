package main

import (
	"errors"
	"flag"
	"fmt"
	"go_server/cli"
	"go_server/client_conn"
	"go_server/config"
	serverError "go_server/error"
	"go_server/game_conn"
	"go_server/helper"
	"go_server/logger"
	"go_server/protocol"
	"go_server/session"
	"io"
	"net"
	"os"
	"os/signal"
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
	serverOptions, err := config.ParseServerOptions(os.Args[1:])
	if err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return
		}
		fmt.Fprintln(os.Stderr, "Invalid arguments:", err)
		os.Exit(2)
	}

	logFile, logErr := os.Create("app.log")
	if logErr != nil {
		logger.AppLogger.Error("Log file error: %v", logErr)
		return
	}
	defer logFile.Close()
	commandReader, cliErr := cli.NewReader()
	if cliErr != nil {
		fmt.Fprintln(os.Stderr, "CLI initialization error:", cliErr)
		return
	}
	logger.AppLogger.SetOutputs(
		io.MultiWriter(commandReader.Stdout(), logger.WithoutANSI(logFile)),
		io.MultiWriter(commandReader.Stderr(), logger.WithoutANSI(logFile)),
	)

	validProtocol := false
	for n := range config.SupportedProtocols {
		if config.SupportedProtocols[n] == config.ProtocolVersion {
			validProtocol = true
		}
	}
	if !validProtocol {
		logger.AppLogger.Error(fmt.Sprintf("Invalid protocol: %d", config.ProtocolVersion))
		os.Exit(int(serverError.CodeProtocolError))
	}

	quit := make(chan struct{})
	var stopOnce sync.Once

	listener, listenErr := net.Listen("tcp", serverOptions.GoServerAddress())
	if listenErr != nil {
		if errors.Is(listenErr, syscall.EADDRINUSE) {
			if owner := helper.PortOwner(serverOptions.GoServerPort); owner != "" {
				listenErr = fmt.Errorf("%w by %s", listenErr, owner)
			}
		}
		logger.AppLogger.Error(fmt.Sprint(listenErr))
		os.Exit(int(serverError.CodeListenerError))
	}
	defer listener.Close()

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	defer signal.Stop(signals)

	go func() {
		<-signals
		shutdownServer(quit, listener, &stopOnce)
	}()

	gameServerManager := &game_conn.GameServerManager{}
	connectionManager := session.NewConnectionManager()
	room := session.NewRoom()

	go cli.Run(commandReader, connectionManager, room, gameServerManager, func() {
		shutdownServer(quit, listener, &stopOnce)
	})
	go connectionManager.RunFloodPointDecay(quit)
	go gameServerManager.HandleGameServer(
		quit,
		serverOptions.RustServerAddress(),
		room.ReconnectPlayersToGameServer,
		room.RouteCommand,
		room.BroadcastEvent,
	)

	logger.AppLogger.Info("TCP server started on %s", net.JoinHostPort(helper.GetServerIP(), fmt.Sprint(serverOptions.GoServerPort)))

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

		client := session.NewClient(conn, room)
		if err := connectionManager.Subscribe(client); err != nil {
			logger.AppLogger.Error("%s Connection rejected: %v", client.Id, err)
			response := protocol.ResponseTooManyRequests
			if errors.Is(err, serverError.ErrMaxConnection) {
				response = protocol.ResponseRoomFull
			}
			if writeErr := client.Write(response); writeErr != nil {
				logger.AppLogger.Error("%s Rejection write error: %v", client.Id, writeErr)
			}
			_ = conn.Close()
			continue
		}

		go func() {
			defer connectionManager.Release(client)
			client_conn.HandleClient(client, gameServerManager, connectionManager)
		}()
	}
}
