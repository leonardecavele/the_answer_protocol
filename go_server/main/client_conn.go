package main

import (
	"bufio"
	"fmt"
	"net"
	"time"
)

import (
	constants "go_server/constants"
)

func handle_client(conn net.Conn, i int, rust_server *RustServer) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	for {
		_, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println(err)
			return
		}
		
		
		fmt.Printf("[%v] Received PING from client %d\n", time.Now().Format(constants.LogFormat), i)
		fmt.Printf("[%v] Sending PING to Rust\n", time.Now().Format(constants.LogFormat))
		err = rust_server.write("PING")
		if err != nil {
			fmt.Println("Rust send error:", err)
			return
		}
	}
}
