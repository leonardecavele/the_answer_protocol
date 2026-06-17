package protocol

import "strings"

type EventBatch struct {
	Player         string   `json:"player"`
	IgnoredPlayers []string `json:"ignored_players"`
	Events         []Event  `json:"events"`
}

type Event struct {
	EmittedBy string `json:"emitted_by"`
	EventName string `json:"event_name"`
	Data      any    `json:"data"`
}

func (event Event) IsValid() bool {
	return strings.TrimSpace(event.EmittedBy) != "" && strings.TrimSpace(event.EventName) != ""
}

func (eventBatch EventBatch) IsValid() bool {
	if len(eventBatch.Events) == 0 {
		return false
	}

	for _, event := range eventBatch.Events {
		if !event.IsValid() {
			return false
		}
	}

	return true
}
