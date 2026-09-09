package cli

import (
	"bufio"
	"fmt"
	"go_server/game_conn"
	"go_server/helper"
	"go_server/logger"
	"go_server/protocol"
	"go_server/session"
	"os"
	"strconv"
	"strings"
	"time"
)

type serverCLI struct {
	connectionManager *session.ConnectionManager
	room              *session.Room
	gameServerManager *game_conn.GameServerManager
	shutdown          func()
	startedAt         time.Time
}

func Run(
	connectionManager *session.ConnectionManager,
	room *session.Room,
	gameServerManager *game_conn.GameServerManager,
	shutdown func(),
) {
	console := serverCLI{
		connectionManager: connectionManager,
		room:              room,
		gameServerManager: gameServerManager,
		shutdown:          shutdown,
		startedAt:         time.Now(),
	}
	console.readCommands()
}

func (console *serverCLI) readCommands() {
	scanner := bufio.NewScanner(os.Stdin)
	interactive := isInteractive(os.Stdin)
	if interactive {
		logger.AppLogger.EnablePrompt(prompt, os.Stdout)
		defer logger.AppLogger.DisablePrompt()
	}
	for {
		if interactive {
			logger.AppLogger.PrintPrompt()
		}
		if !scanner.Scan() {
			break
		}
		if interactive {
			logger.AppLogger.ConsumePrompt()
		}

		command, arguments := splitCommand(scanner.Text())
		if command == "" {
			continue
		}
		if console.handleCommand(command, arguments) {
			return
		}
	}

	if err := scanner.Err(); err != nil {
		logger.AppLogger.Error("CLI input error: %v", err)
	}
}

func isInteractive(input *os.File) bool {
	info, err := input.Stat()
	return err == nil && info.Mode()&os.ModeCharDevice != 0
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

func (console *serverCLI) handleCommand(command string, arguments string) bool {
	if _, ok := quitCommands[command]; ok {
		if !hasNoArguments(command, arguments) {
			return false
		}
		console.shutdown()
		return true
	}

	switch command {
	case CommandBan:
		handleIPCommand(arguments, "ban <ip>", console.connectionManager.BanIP, "banned")
	case CommandUnban:
		handleIPCommand(arguments, "deban <ip>", console.connectionManager.UnbanIP, "unbanned")
	case CommandStatus:
		console.showStatus(arguments)
	case CommandClients:
		console.showClients(arguments)
	case CommandKick:
		console.kick(arguments)
	case CommandBans:
		console.showBans(arguments)
	case CommandFlood:
		console.showFlood(arguments)
	case CommandBroadcast:
		console.broadcastCommand(arguments)
	case CommandGroups:
		console.showGroups(arguments)
	case CommandGameServer:
		console.handleGameServer(arguments)
	case CommandShutdown:
		return console.scheduleShutdown(arguments)
	case CommandHelp:
		if hasNoArguments(CommandHelp, arguments) {
			logger.AppLogger.Console(helpMessage)
		}
	default:
		logger.AppLogger.Error("Unknown CLI command %q. Enter help to list commands.", command)
	}
	return false
}

func hasNoArguments(command string, arguments string) bool {
	if arguments == "" {
		return true
	}
	logger.AppLogger.Error("Usage: %s", command)
	return false
}

func handleIPCommand(arguments string, usage string, action func(string) bool, pastTense string) {
	fields := strings.Fields(arguments)
	if len(fields) != 1 {
		logger.AppLogger.Error("Usage: %s", usage)
		return
	}
	if !helper.IsValidIP(fields[0]) {
		logger.AppLogger.Error("Invalid IP address: %s", fields[0])
		return
	}

	ip := helper.NormalizeIP(fields[0])
	if !action(ip) {
		logger.AppLogger.Error("Could not update IP address: %s", ip)
		return
	}
	logger.AppLogger.Info("IP %s: %s", pastTense, ip)
}

func (console *serverCLI) showStatus(arguments string) {
	if !hasNoArguments(CommandStatus, arguments) {
		return
	}

	gameServerStatus := "disconnected"
	if console.gameServerManager.IsConnected() {
		gameServerStatus = "connected"
	}
	logger.AppLogger.Info(
		"Server status: uptime=%s connections=%d clients=%d groups=%d bans=%d game_server=%s",
		time.Since(console.startedAt).Round(time.Second),
		console.connectionManager.Count(),
		console.room.Count(),
		len(console.room.Groups()),
		len(console.connectionManager.BannedIPs()),
		gameServerStatus,
	)
}

func (console *serverCLI) showClients(arguments string) {
	if !hasNoArguments(CommandClients, arguments) {
		return
	}

	clients := console.connectionManager.Clients()
	if len(clients) == 0 {
		logger.AppLogger.Info("No clients connected")
		return
	}
	for _, client := range clients {
		username := client.Username
		if username == "" {
			username = "-"
		}
		logger.AppLogger.Info(
			"Client: username=%s ip=%s state=%s connected=%s",
			username,
			client.IP,
			client.State,
			client.ConnectedFor,
		)
	}
}

func (console *serverCLI) kick(arguments string) {
	fields := strings.Fields(arguments)
	if len(fields) != 1 {
		logger.AppLogger.Error("Usage: kick <username>")
		return
	}

	username := strings.ToUpper(fields[0])
	if !console.room.Disconnect(username) {
		logger.AppLogger.Error("Client not found: %s", username)
		return
	}
	logger.AppLogger.Info("Client kicked: username=%s", username)
}

func (console *serverCLI) showBans(arguments string) {
	if !hasNoArguments(CommandBans, arguments) {
		return
	}

	bans := console.connectionManager.BannedIPs()
	if len(bans) == 0 {
		logger.AppLogger.Info("No banned IP addresses")
		return
	}
	for _, ban := range bans {
		logger.AppLogger.Info("Banned IP: ip=%s points=%d/%d", ban.IP, ban.Points, ban.MaxPoints)
	}
}

func (console *serverCLI) showFlood(arguments string) {
	fields := strings.Fields(arguments)
	if len(fields) != 1 || !helper.IsValidIP(fields[0]) {
		logger.AppLogger.Error("Usage: flood <ip>")
		return
	}

	info, ok := console.connectionManager.FloodInfo(fields[0])
	if !ok {
		logger.AppLogger.Error("Invalid IP address: %s", fields[0])
		return
	}
	logger.AppLogger.Info(
		"IP flood: ip=%s points=%d/%d banned=%t",
		info.IP,
		info.Points,
		info.MaxPoints,
		info.Banned,
	)
}

func (console *serverCLI) broadcastCommand(message string) {
	if message == "" {
		logger.AppLogger.Error("Usage: broadcast <message>")
		return
	}

	console.broadcast(message)
	logger.AppLogger.Info("Broadcast sent: %s", message)
}

func (console *serverCLI) broadcast(message string) {
	console.room.BroadcastEvent(protocol.EventBatch{
		Events: []protocol.Event{
			{
				EventName: "BROADCAST",
				Data:      message,
			},
		},
	})
}

func (console *serverCLI) showGroups(arguments string) {
	if !hasNoArguments(CommandGroups, arguments) {
		return
	}

	groups := console.room.Groups()
	if len(groups) == 0 {
		logger.AppLogger.Info("No active groups")
		return
	}
	for _, group := range groups {
		logger.AppLogger.Info(
			"Group: id=%s leader=%s members=%s",
			group.ID,
			group.Leader,
			strings.Join(group.Members, ","),
		)
	}
}

func (console *serverCLI) handleGameServer(arguments string) {
	fields := strings.Fields(arguments)
	if len(fields) != 1 || strings.ToLower(fields[0]) != GameServerReconnect {
		logger.AppLogger.Error("Usage: gameserver reconnect")
		return
	}

	if err := console.gameServerManager.Reconnect(); err != nil {
		logger.AppLogger.Error("Game server reconnect error: %v", err)
		return
	}
	logger.AppLogger.Info("Game server reconnect requested")
}

func (console *serverCLI) scheduleShutdown(arguments string) bool {
	if arguments == "" {
		logger.AppLogger.Info("Server shutdown requested")
		console.shutdown()
		return true
	}

	fields := strings.Fields(arguments)
	if len(fields) != 1 {
		logger.AppLogger.Error("Usage: shutdown [seconds]")
		return false
	}
	seconds, err := strconv.ParseUint(fields[0], 10, 31)
	if err != nil {
		logger.AppLogger.Error("Usage: shutdown [seconds]")
		return false
	}
	if seconds == 0 {
		logger.AppLogger.Info("Server shutdown requested")
		console.shutdown()
		return true
	}

	message := fmt.Sprintf("Server shutdown in %d seconds", seconds)
	console.broadcast(message)
	logger.AppLogger.Info("Server shutdown scheduled in %d seconds", seconds)
	time.AfterFunc(time.Duration(seconds)*time.Second, console.shutdown)
	return true
}
