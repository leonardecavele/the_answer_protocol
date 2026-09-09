package cli

const (
	CommandBan          = "ban"
	CommandUnban        = "deban"
	CommandStatus       = "status"
	CommandClients      = "clients"
	CommandKick         = "kick"
	CommandBans         = "bans"
	CommandFlood        = "flood"
	CommandBroadcast    = "broadcast"
	CommandShutdown     = "shutdown"
	CommandGameServer   = "gameserver"
	CommandGroups       = "groups"
	CommandHelp         = "help"
	CommandQuit         = "quit"
	CommandExit         = "exit"
	CommandQuitShortcut = "q"

	GameServerReconnect = "reconnect"
)

var quitCommands = map[string]struct{}{
	CommandQuit:         {},
	CommandExit:         {},
	CommandQuitShortcut: {},
}

const helpMessage = `Available commands:
  ban <ip>                 Ban an IP address and disconnect its active clients
  deban <ip>               Unban an IP address and clear all of its flood state
  status                   Show the server status
  clients                  List connected clients
  kick <username>          Disconnect a client
  bans                     List banned IP addresses
  flood <ip>               Show the flood state of an IP address
  broadcast <message>      Send EVT BROADCAST <message> to every client
  groups                   List active groups
  gameserver reconnect     Force the Rust game server to reconnect
  shutdown [seconds]       Stop the server immediately or after a delay
  help                     Show this help
  quit                     Stop the server (aliases: exit, q)
`
