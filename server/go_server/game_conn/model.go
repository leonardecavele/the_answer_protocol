package game_conn

type CommandToGameServer struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	Arguments string `json:"arguments"`
}

type CommandFromGameServer struct {
	Player    string `json:"player"`
	Command   string `json:"command"`
	ErrorCode int    `json:"error_code"`
	Data      string `json:"data"`
}

type QuestionToGameServer struct {
	Question string `json:"question"`
	Data     string `json:"data"`
	Id       string `json:"id"`
}

type AnswerFromGameServer struct {
	Question string `json:"question"`
	Data     string `json:"data"`
	Id       string `json:"id"`
}
