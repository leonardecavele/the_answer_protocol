package protocol

type Event struct {
	Player         string   `json:"player"`
	IgnoredPlayers []string `json:"ignored_players"`
	EmitedBy       string   `json:"emited_by"`
	EventName      string   `json:"event_name"`
	Data           any      `json:"data"`
}
