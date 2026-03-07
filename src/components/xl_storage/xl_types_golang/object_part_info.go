package xl_types_golang

import (
	"time"
)

//go:generate msgp

// ObjectPartInfo Info of each part kept in the multipart metadata
// file after CompleteMultipartUpload() is called.
type ObjectPartInfo struct {
	ETag       string            `json:"etag,omitempty" msg:"e"`
	Number     int               `json:"number" msg:"n"`
	Size       int64             `json:"size" msg:"s"`        // Size of the part on the disk.
	ActualSize int64             `json:"actualSize" msg:"as"` // Original size of the part without compression or encryption bytes.
	ModTime    time.Time         `json:"modTime" msg:"mt"`    // Date and time at which the part was uploaded.
	Index      []byte            `json:"index,omitempty" msg:"i,omitempty"`
	Checksums  map[string]string `json:"crc,omitempty" msg:"crc,omitempty"`   // Content Checksums
	Error      string            `json:"error,omitempty" msg:"err,omitempty"` // only set while reading part meta from drive.
}
