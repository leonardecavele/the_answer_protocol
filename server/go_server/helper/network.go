package helper

import (
	"bufio"
	"errors"
	serverError "go_server/error"
	"net"
)

func ReadStringWithLimit(reader *bufio.Reader, delimiter byte, maxSize int) (string, error) {
	fragment, err := reader.ReadSlice(delimiter)
	if len(fragment) > maxSize {
		return "", serverError.ErrReadStringTooLong
	}
	if !errors.Is(err, bufio.ErrBufferFull) {
		return string(fragment), err
	}

	message := append([]byte(nil), fragment...)

	for {
		fragment, err = reader.ReadSlice(delimiter)
		if len(fragment) > maxSize-len(message) {
			return string(message), serverError.ErrReadStringTooLong
		}
		message = append(message, fragment...)

		if err == nil {
			return string(message), nil
		}
		if !errors.Is(err, bufio.ErrBufferFull) {
			return string(message), err
		}
	}
}

func GetServerIP() string {
	interfaces, err := net.Interfaces()
	if err != nil {
		return "127.0.0.1"
	}

	for _, iface := range interfaces {
		if iface.Flags&net.FlagUp == 0 || iface.Flags&net.FlagLoopback != 0 {
			continue
		}

		addrs, err := iface.Addrs()
		if err != nil {
			continue
		}

		for _, addr := range addrs {
			ipNet, ok := addr.(*net.IPNet)
			if !ok {
				continue
			}

			ip := ipNet.IP.To4()
			if ip != nil {
				return ip.String()
			}
		}
	}

	return "127.0.0.1"
}
