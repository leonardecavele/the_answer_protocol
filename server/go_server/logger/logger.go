package logger

import (
	"go_server/config"
	"log"
	"os"
	"time"
)

const (
	colorRed   = "\033[31m"
	colorGreen = "\033[32m"
	colorReset = "\033[0m"
)

type Logger struct {
	*log.Logger
}

func (l Logger) Info(format string, v ...any) {
	l.Printf(time.Now().Format(config.LogFormat)+" "+colorGreen+"INFO"+colorReset+" "+format, v...)
}

func (l Logger) Error(format string, v ...any) {
	l.Printf(time.Now().Format(config.LogFormat)+" "+colorRed+"ERROR"+colorReset+" "+format, v...)
}

var AppLogger = Logger{log.New(os.Stdout, "", 0)}
