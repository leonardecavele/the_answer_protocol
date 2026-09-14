package helper

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

func PortOwner(port int) string {
	inode := listeningSocketInode(port)
	if inode == "" {
		return ""
	}

	processes, err := os.ReadDir("/proc")
	if err != nil {
		return ""
	}
	for _, process := range processes {
		pid := process.Name()
		if _, err := strconv.Atoi(pid); err != nil {
			continue
		}
		fds, err := os.ReadDir(filepath.Join("/proc", pid, "fd"))
		if err != nil {
			continue
		}
		for _, fd := range fds {
			target, err := os.Readlink(filepath.Join("/proc", pid, "fd", fd.Name()))
			if err == nil && target == "socket:["+inode+"]" {
				name, _ := os.ReadFile(filepath.Join("/proc", pid, "comm"))
				if name := strings.TrimSpace(string(name)); name != "" {
					return fmt.Sprintf("PID %s (%s)", pid, name)
				}
				return "PID " + pid
			}
		}
	}
	return ""
}

func listeningSocketInode(port int) string {
	for _, path := range []string{"/proc/net/tcp", "/proc/net/tcp6"} {
		file, err := os.Open(path)
		if err != nil {
			continue
		}
		scanner := bufio.NewScanner(file)
		for scanner.Scan() {
			fields := strings.Fields(scanner.Text())
			if len(fields) < 10 || fields[3] != "0A" {
				continue
			}
			separator := strings.LastIndexByte(fields[1], ':')
			if separator < 0 {
				continue
			}
			parsedPort, err := strconv.ParseInt(fields[1][separator+1:], 16, 32)
			if err == nil && int(parsedPort) == port {
				file.Close()
				return fields[9]
			}
		}
		file.Close()
	}
	return ""
}
