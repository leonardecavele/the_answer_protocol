package rust_conn

import (
	"net"
	"sync"
)

type RustServer struct {
	Conn       net.Conn
	PrintMutex sync.Mutex
}

type RustCommand struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	Arguments string `json:"arguments"`
}
