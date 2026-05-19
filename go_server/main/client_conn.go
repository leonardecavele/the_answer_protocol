package main

import (
	"fmt"
	"net"
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
