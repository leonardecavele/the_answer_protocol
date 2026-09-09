package logger

import (
	"fmt"
	"go_server/config"
	"io"
	"log"
	"os"
	"sync"
	"time"
)

type Logger struct {
	info         *log.Logger
	error        *log.Logger
	mutex        sync.Mutex
	prompt       string
	promptOutput io.Writer
	promptActive bool
	promptShown  bool
}

func (l *Logger) Info(format string, v ...any) {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.clearPrompt()
	l.info.Printf(time.Now().Format(config.LogFormat)+" "+colorGreen+"INFO"+colorReset+" "+format, v...)
	l.showPrompt()
}

func (l *Logger) Error(format string, v ...any) {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.clearPrompt()
	l.error.Printf(time.Now().Format(config.LogFormat)+" "+colorRed+"ERROR"+colorReset+" "+format, v...)
	l.showPrompt()
}

func (l *Logger) SetOutputs(infoOutput, errorOutput io.Writer) {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.info.SetOutput(infoOutput)
	l.error.SetOutput(errorOutput)
}

func (l *Logger) EnablePrompt(prompt string, output io.Writer) {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.prompt = prompt
	l.promptOutput = output
	l.promptActive = prompt != "" && output != nil
	l.promptShown = false
}

func (l *Logger) DisablePrompt() {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.clearPrompt()
	l.prompt = ""
	l.promptOutput = nil
	l.promptActive = false
	l.promptShown = false
}

func (l *Logger) PrintPrompt() {
	l.mutex.Lock()
	defer l.mutex.Unlock()
	l.showPrompt()
}

func (l *Logger) ConsumePrompt() {
	l.mutex.Lock()
	l.promptShown = false
	l.mutex.Unlock()
}

func (l *Logger) Console(message string) {
	l.mutex.Lock()
	defer l.mutex.Unlock()

	l.clearPrompt()
	output := l.promptOutput
	if output == nil {
		output = os.Stdout
	}
	fmt.Fprint(output, message)
}

func (l *Logger) clearPrompt() {
	if !l.promptActive || !l.promptShown {
		return
	}
	fmt.Fprint(l.promptOutput, "\r\033[2K")
	l.promptShown = false
}

func (l *Logger) showPrompt() {
	if !l.promptActive || l.promptShown {
		return
	}
	fmt.Fprint(l.promptOutput, l.prompt)
	l.promptShown = true
}

var AppLogger = Logger{
	info:  log.New(os.Stdout, "", 0),
	error: log.New(os.Stderr, "", 0),
}
