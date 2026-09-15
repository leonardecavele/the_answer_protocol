package helper

import (
	"net"
	"strings"
)

func IsValidIP(value string) bool {
	return NormalizeIP(value) != ""
}

func NormalizeIP(value string) string {
	ip := net.ParseIP(strings.TrimSpace(value))
	if ip == nil {
		return ""
	}
	return ip.String()
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
