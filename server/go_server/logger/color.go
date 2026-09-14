package logger

import (
	"io"
	"regexp"
)

const (
	colorRed   = "\033[31m"
	colorGreen = "\033[32m"
	colorReset = "\033[0m"
)

var ansiSequence = regexp.MustCompile(`\x1b\[[0-?]*[ -/]*[@-~]`)

type plainWriter struct {
	output io.Writer
}

func (writer plainWriter) Write(data []byte) (int, error) {
	clean := ansiSequence.ReplaceAll(data, nil)
	_, err := writer.output.Write(clean)
	return len(data), err
}

func WithoutANSI(output io.Writer) io.Writer {
	return plainWriter{output: output}
}
