package main

import (
	"fmt"
	"net"
	"sync"
	"time"
)

type RustServer struct {
	conn  net.Conn
	mutex sync.Mutex
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
