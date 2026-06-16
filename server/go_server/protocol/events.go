package protocol

var (
	// Room.
	EventRoomPresenceEnter = "EVT ROOM PRESENCE ENTER <username>"
	EventRoomPresenceLeave = "EVT ROOM PRESENCE LEAVE <username>"
	EventRoomChat          = "EVT ROOM CHAT <username> <message>"

	// Global.
	EventGlobalChat = "EVT GLOBAL CHAT <username> <message>"

	// Group.
	EventGroupInvite = "EVT GROUP INVITE <leader>"
	EventGroupJoin   = "EVT GROUP JOIN <username>"
	EventGroupLeave  = "EVT GROUP LEAVE <username>"
	EventGroupChat   = "EVT GROUP CHAT <username> <message>"

	// Stats.
	EventStatsPlayers = "EVT STATS players=<count>"
)
