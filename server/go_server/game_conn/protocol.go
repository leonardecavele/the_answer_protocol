package game_conn

import (
	"bufio"
	"net"
	"sync"
)

type GameServer struct {
	Conn       net.Conn
	Writer     *bufio.Writer
	PrintMutex sync.Mutex
}

type CommandToGameServer struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	Arguments string `json:"arguments"`
}

type CommandFromGameServer struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	ErrorCode int    `json:"error_code"`
	Data      string `json:"data"`
}

type EventFromGameServer struct {
	Player    string `json:"player"`
	EventName string `json:"event_name"`
	Data      any    `json:"data"`
}
