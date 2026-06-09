package rust_conn

import (
	"bufio"
	"fmt"
	"net"
	"strings"
	"time"
)

import (
	"go_server/config"
)

func ConnectToRust(addr string) *RustServer {
	for {
		conn, err := net.Dial("tcp", addr)
		if err == nil {
			fmt.Println("Connected to Rust server")
			return &RustServer{Conn: conn}
		}

		fmt.Println("Waiting for Rust server:", err)
		time.Sleep(time.Second)
	}
}

func (rustServer *RustServer) Write(message string) error {
	rustServer.PrintMutex.Lock()
	defer rustServer.PrintMutex.Unlock()

	_, err := fmt.Fprintf(rustServer.Conn, "%s\n", message)
	return err
}

func (rustServer *RustServer) Read() {
	reader := bufio.NewReader(rustServer.Conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println("Rust connection closed:", err)
			return
		}

		message = strings.TrimSpace(message)
		time := time.Now().Format(config.LogFormat)
		fmt.Printf("[%v] Received %s from Rust\n", time, message)
	}
}
