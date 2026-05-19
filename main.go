package main

import (
	"fmt"
	"net"
	"os"
)

type ErrorCode int

const (
	NoError ErrorCode = iota
	ListenerError
)

func handle_client(conn net.Conn, i int) {
	defer conn.Close()
	buffer := make([]byte, 1024)

	for {
		_, err := conn.Read(buffer)
		if err != nil {
			fmt.Println(err)
			return
		}
		fmt.Printf("Received ping from client %d\n", i)
	}
}

func main() {

	listener, err := net.Listen("tcp", ":38800")
	if err != nil {
		os.Exit(int(ListenerError))
	}

	fmt.Println("TCP server started on 38800")

	i := 0
	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Accept error:", err)
			continue
		}

		go handle_client(conn, i)
		i++
	}
}
