package rust_conn

import (
	"net"
	"sync"
)

type RustServer struct {
	Conn       net.Conn
	PrintMutex sync.Mutex
}

type CommandToRust struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	Arguments string `json:"arguments"`
}

type CommandFromRust struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	ErrorCode int    `json:"error_code"`
	Value     string `json:"value"`
}

type EventFromRust struct {
	Player    string `json:"player"`
	EventName string `json:"event_name"`
	Value     string `json:"value"`
}
