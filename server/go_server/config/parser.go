package config

import (
	"errors"
	"flag"
	"fmt"
	"net"
	"strconv"
	"strings"
)

type ServerOptions struct {
	RustServerIP   string
	RustServerPort int
	GoServerPort   int
}

func DefaultServerOptions() ServerOptions {
	return ServerOptions{
		RustServerIP:   GameServerIP,
		RustServerPort: GameServerPort,
		GoServerPort:   GoServerPort,
	}
}

func ParseServerOptions(args []string) (ServerOptions, error) {
	options := DefaultServerOptions()
	flags := flag.NewFlagSet("go_server", flag.ContinueOnError)

	flags.StringVar(&options.RustServerIP, "rust-server-ip", options.RustServerIP, "IP or hostname of the Rust server")
	flags.IntVar(&options.RustServerPort, "rust-server-port", options.RustServerPort, "port of the Rust server")
	flags.IntVar(&options.GoServerPort, "go-server-port", options.GoServerPort, "listening port of the Go server")

	if err := flags.Parse(args); err != nil {
		return ServerOptions{}, err
	}
	if flags.NArg() != 0 {
		return ServerOptions{}, fmt.Errorf("unexpected positional arguments: %s", strings.Join(flags.Args(), " "))
	}
	if strings.TrimSpace(options.RustServerIP) == "" {
		return ServerOptions{}, errors.New("rust-server-ip cannot be empty")
	}
	if err := validatePort("rust-server-port", options.RustServerPort); err != nil {
		return ServerOptions{}, err
	}
	if err := validatePort("go-server-port", options.GoServerPort); err != nil {
		return ServerOptions{}, err
	}

	return options, nil
}

func validatePort(name string, port int) error {
	if port < 1 || port > 65535 {
		return fmt.Errorf("%s must be between 1 and 65535", name)
	}
	return nil
}

func (options ServerOptions) RustServerAddress() string {
	return net.JoinHostPort(options.RustServerIP, strconv.Itoa(options.RustServerPort))
}

func (options ServerOptions) GoServerAddress() string {
	return net.JoinHostPort("", strconv.Itoa(options.GoServerPort))
}
