package session

import "time"

type rateWindow struct {
	startedAt time.Time
	count     int
}

func (window *rateWindow) allow(now time.Time, limit int, duration time.Duration) bool {
	if window.startedAt.IsZero() || now.Sub(window.startedAt) >= duration {
		window.startedAt = now
		window.count = 1
		return true
	}
	if window.count >= limit {
		return false
	}
	window.count++
	return true
}

func (window *rateWindow) expired(now time.Time, duration time.Duration) bool {
	return !window.startedAt.IsZero() && now.Sub(window.startedAt) >= duration
}
