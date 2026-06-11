package main

import (
	"bufio"
	"net"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"sync"
	"syscall"
)

import (
	"go_server/client_conn"
	"go_server/config"
	"go_server/error"
	"go_server/logger"
	"go_server/rust_conn"
)

func getServerIP() string {
	interfaces, err := net.Interfaces()
	if err != nil {
		return "127.0.0.1"
	}

	for _, iface := range interfaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}

		addrs, err := iface.Addrs()
		if err != nil {
			continue
		}

		for _, addr := range addrs {
			ipNet, ok := addr.(*net.IPNet)
			if !ok {
				continue
			}

			ip := ipNet.IP.To4()
			if ip != nil {
				return ip.String()
			}
		}
	}

	return "127.0.0.1"
}

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

	stopServer := func() {
		shutdownServer(quit, listener, &stopOnce)
	}

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	defer signal.Stop(signals)

	go func() {
		<-signals
		stopServer()
	}()

	go func() {
		scanner := bufio.NewScanner(os.Stdin)

		for scanner.Scan() {
			input := strings.ToLower(strings.TrimSpace(scanner.Text()))

			if _, ok := config.QuitCommands[input]; ok {
				stopServer()
				return
			}
		}
	}()

	rustServer := rust_conn.ConnectToRust(config.RustServerIP+":"+strconv.Itoa(config.RustServerPort), quit)
	if rustServer == nil {
		return
	}
	defer rustServer.Close()

	newListener, listenErr := net.Listen("tcp", ":"+strconv.Itoa(config.GoServerPort))
	if listenErr != nil {
		os.Exit(int(error.ListenerError))
	}
	listener = newListener
	defer listener.Close()

	go rustServer.Read(quit, client_conn.RouteCommand, client_conn.RouteEvent)

	logger.AppLogger.Info("TCP server started on " + getServerIP() + ":" + strconv.Itoa(config.GoServerPort))

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

		go client_conn.HandleClient(client_conn.NewClient(conn), rustServer, nil)
	}

	os.Exit(int(error.NoError))
}
