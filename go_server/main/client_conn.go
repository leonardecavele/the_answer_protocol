package main

import (
	"bufio"
	"fmt"
	"net"
	"time"
)

func handle_client(conn net.Conn, i int, rust_server *RustServer) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println(err)
			return
		}

		if message == "stop" {
			return
		}

		fmt.Printf("Received ping from client %d\n", i)
		fmt.Printf("[%v] Sending PING to Rust\n", time.Now())
		err = rust_server.write("PING")
		if err != nil {
			fmt.Println("Rust send error:", err)
			return
		}
	}
}
