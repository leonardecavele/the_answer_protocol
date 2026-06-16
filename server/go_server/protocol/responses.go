package protocol

import (
	"go_server/config"
	"strconv"
)

var (
	// Connection.
	ResponseHello     = "OK hello proto=" + strconv.Itoa(config.ProtocolVersion)
	ResponseConnected = "OK connected"
	ResponseBye       = "OK bye"
)
