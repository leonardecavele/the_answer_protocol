package protocol

// Room
var (
	EventRoomPresenceEnter = "EVT ROOM PRESENCE ENTER <username>"
	EventRoomPresenceLeave = "EVT ROOM PRESENCE LEAVE <username>"
	EventRoomChat          = "EVT ROOM CHAT <username> <message>"
)

// Global
var (
	EventGlobalChat = "EVT GLOBAL CHAT <username> <message>"
)

// Group
var (
	EventGroupInvite = "EVT GROUP INVITE <leader>"
	EventGroupJoin   = "EVT GROUP JOIN <username>"
	EventGroupLeave  = "EVT GROUP LEAVE <username>"
	EventGroupChat   = "EVT GROUP CHAT <username> <message>"
)

// Stats
var (
	EventStatsPlayers = "EVT STATS players=<count>"
)
