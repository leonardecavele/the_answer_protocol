package protocol

import "encoding/json"

func FormatEvent(event Event) (string, error) {
	message := "EVT " + event.EventName + " " + event.EmittedBy
	if event.Data == nil {
		return message, nil
	}

	data, ok := event.Data.(string)
	if !ok {
		dataBytes, err := json.Marshal(event.Data)
		if err != nil {
			return "", err
		}
		data = string(dataBytes)
	}

	if data == "" {
		return message, nil
	}
	return message + " " + data, nil
}
