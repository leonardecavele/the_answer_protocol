package main

import (
	"bufio"
	"fmt"
	"net"
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
		err = rust_server.write(fmt.Sprintf("PING %d", i))
		if err != nil {
			fmt.Println("Rust send error:", err)
			return
		}
	}
}
