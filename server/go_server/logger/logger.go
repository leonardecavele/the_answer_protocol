package logger

import (
	"log"
	"os"
)

const (
	colorRed    = "\033[31m"
	colorYellow = "\033[33m"
	colorReset  = "\033[0m"
)

type Logger struct {
	*log.Logger
}

func (l Logger) Info(format string, v ...any) {
	l.Printf(colorYellow+"INFO"+colorReset+" | "+format, v...)
}

func (l Logger) Error(format string, v ...any) {
	l.Printf(colorRed+"ERROR"+colorReset+" | "+format, v...)
}

var AppLogger = Logger{log.New(os.Stdout, "", 0)}
