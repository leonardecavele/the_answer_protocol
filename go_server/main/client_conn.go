package main

import (
	"fmt"
	"net"
)

func handle_client(conn net.Conn, i int, rust_server *RustServer) {
	defer conn.Close()
	buffer := make([]byte, 1024)

	for {
		_, err := conn.Read(buffer)
		if err != nil {
			fmt.Println(err)
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
