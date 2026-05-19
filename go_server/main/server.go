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

	fmt.Println("TCP server started on 38800")

	i := 0
	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Accept error:", err)
			continue
		}

		go handle_client(conn, i, rust_server)
		i++
	}
}
