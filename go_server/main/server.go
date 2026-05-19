package main

import (
	"fmt"
	"net"
	"os"
)

import (
	error "go_server/error"
)

func main() {

	rust_server := connect_to_rust("127.0.0.1:38801")
	defer rust_server.conn.Close()
	go rust_server.read_loop()

	listener, err := net.Listen("tcp", ":38800")
	if err != nil {
		os.Exit(int(error.ListenerError))
	}
	defer listener.Close()
	fmt.Println("TCP server started on 38800")

	for i := 0; true; i++ {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Accept error:", err)
			continue
		}

		go handle_client(conn, i, rust_server)
	}
}
