package protocol

import (
	"go_server/config"
	"strconv"
)

// Connection
var (
	ResponseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion)
	ResponseConnected = "OK connected"
	ResponseBye       = "OK bye"
)
