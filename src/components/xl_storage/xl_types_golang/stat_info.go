package xl_types_golang

import "time"

//go:generate msgp

// StatInfo - carries stat information of the object.
type StatInfo struct {
	Size    int64     `json:"size"`    // Size of the object `xl.meta`.
	ModTime time.Time `json:"modTime"` // ModTime of the object `xl.meta`.
	Name    string    `json:"name"`
	Dir     bool      `json:"dir"`
	Mode    uint32    `json:"mode"`
}
