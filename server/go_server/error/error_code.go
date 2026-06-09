package error

type ErrorCode int

const (
	NoError ErrorCode = iota
	ListenerError
)
