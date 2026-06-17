package error

type CodeError int

const (
	CodeNoError CodeError = iota
	CodeListenerError
)
