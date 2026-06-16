package protocol

type Event struct {
	Player    string `json:"player"`
	EventName string `json:"event_name"`
	Data      any    `json:"data"`
}
