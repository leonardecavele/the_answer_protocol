package cli

import (
	"errors"
	"io"
	"os"
	"strings"
	"time"

	"github.com/ergochat/readline"
	"golang.org/x/term"

	"go_server/game_conn"
	"go_server/logger"
	"go_server/session"
)

type serverCLI struct {
	clientConnectionManager *session.ClientConnectionManager
	room                    *session.Room
	gameServerManager       *game_conn.GameServerManager
	shutdown                func()
	startedAt               time.Time
}

func Run(
	reader *readline.Instance,
	clientConnectionManager *session.ClientConnectionManager,
	room *session.Room,
	gameServerManager *game_conn.GameServerManager,
	shutdown func(),
) {
	defer reader.Close()
	console := serverCLI{
		clientConnectionManager: clientConnectionManager,
		room:                    room,
		gameServerManager:       gameServerManager,
		shutdown:                shutdown,
		startedAt:               time.Now(),
	}
	console.readCommands(reader)
}

func NewReader() (*readline.Instance, error) {
	interactive := term.IsTerminal(int(os.Stdin.Fd())) && term.IsTerminal(int(os.Stdout.Fd()))
	configuredPrompt := ""
	if interactive {
		configuredPrompt = prompt
	}
	return readline.NewEx(&readline.Config{
		Prompt:          configuredPrompt,
		InterruptPrompt: "^C",
		FuncIsTerminal:  func() bool { return interactive },
	})
}

func (console *serverCLI) readCommands(reader *readline.Instance) {
	for {
		line, err := reader.Readline()
		if errors.Is(err, readline.ErrInterrupt) {
			continue
		}
		if errors.Is(err, io.EOF) {
			return
		}
		if err != nil {
			logger.AppLogger.Error("CLI input error: %v", err)
			return
		}

		command, arguments := splitCommand(line)
		if command == "" {
			continue
		}
		if console.handleCommand(command, arguments) {
			return
		}
	}
}

func splitCommand(input string) (string, string) {
	input = strings.TrimSpace(input)
	fields := strings.Fields(input)
	if len(fields) == 0 {
		return "", ""
	}

	command := fields[0]
	arguments := strings.TrimSpace(input[len(command):])
	return strings.ToLower(command), arguments
}
