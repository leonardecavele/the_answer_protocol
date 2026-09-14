package helper

import (
	"crypto/rand"
	"encoding/hex"
)

const idByteLength = 16

func NewID() (string, error) {
	bytes := make([]byte, idByteLength)
	if _, err := rand.Read(bytes); err != nil {
		return "", err
	}

	return hex.EncodeToString(bytes), nil
}

func IsValidID(id string) bool {
	if len(id) != hex.EncodedLen(idByteLength) {
		return false
	}
	_, err := hex.DecodeString(id)
	return err == nil
}
