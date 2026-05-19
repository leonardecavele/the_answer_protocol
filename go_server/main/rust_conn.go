package main

import (
	"bufio"
	"fmt"
	"net"
	"sync"
	"time"
)

type RustServer struct {
	conn        net.Conn
	print_mutex sync.Mutex
}

func connect_to_rust(addr string) *RustServer {
	for {
		conn, err := net.Dial("tcp", addr)
		if err == nil {
			fmt.Println("Connected to Rust server")
			return &RustServer{conn: conn}
		}

		fmt.Println("Waiting for Rust server:", err)
		time.Sleep(time.Second)
	}
}

func (rust_server *RustServer) write(message string) error {
	rust_server.print_mutex.Lock()
	defer rust_server.print_mutex.Unlock()

	_, err := fmt.Fprintf(rust_server.conn, "%s\n", message)
	return err
}

func (rust_server *RustServer) read_loop() {
	reader := bufio.NewReader(rust_server.conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println("Rust connection closed:", err)
			return
		}

		fmt.Printf("[%v] Received %s from Rust", time.Now(), message)
	}
}
