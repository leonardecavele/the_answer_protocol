package logger

import (
	"go_server/config"
	"io"
	"log"
	"os"
	"time"
)

type Logger struct {
	info  *log.Logger
	error *log.Logger
}

func (l Logger) Info(format string, v ...any) {
	l.info.Printf(time.Now().Format(config.LogFormat)+" "+colorGreen+"INFO"+colorReset+" "+format, v...)
}

func (l Logger) Error(format string, v ...any) {
	l.error.Printf(time.Now().Format(config.LogFormat)+" "+colorRed+"ERROR"+colorReset+" "+format, v...)
}

func (l Logger) SetOutputs(infoOutput, errorOutput io.Writer) {
	l.info.SetOutput(infoOutput)
	l.error.SetOutput(errorOutput)
}

var AppLogger = Logger{
	info:  log.New(os.Stdout, "", 0),
	error: log.New(os.Stderr, "", 0),
}
