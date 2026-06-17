package protocol

type Event struct {
	Players        []string `json:"players"`
	IgnoredPlayers []string `json:"ignored_players"`
	EmittedBy      string   `json:"emitted_by"`
	EventName      string   `json:"event_name"`
	Data           any      `json:"data"`
}
