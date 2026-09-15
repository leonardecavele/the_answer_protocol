package helper

import (
	"bufio"
	"errors"
	serverError "go_server/error"
)

func ReadStringWithLimit(reader *bufio.Reader, delimiter byte, maxSize int) (string, error) {
	fragment, err := reader.ReadSlice(delimiter)
	if len(fragment) > maxSize {
		return "", serverError.ErrReadStringTooLong
	}
	if !errors.Is(err, bufio.ErrBufferFull) {
		return string(fragment), err
	}

	message := append([]byte(nil), fragment...)

	for {
		fragment, err = reader.ReadSlice(delimiter)
		if len(fragment) > maxSize-len(message) {
			return string(message), serverError.ErrReadStringTooLong
		}
		message = append(message, fragment...)

		if err == nil {
			return string(message), nil
		}
		if !errors.Is(err, bufio.ErrBufferFull) {
			return string(message), err
		}
	}
}
