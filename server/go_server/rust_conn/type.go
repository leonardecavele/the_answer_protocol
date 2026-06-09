package rust_conn

import (
	"net"
	"sync"
)

type RustServer struct {
	Conn       net.Conn
	PrintMutex sync.Mutex
}
