package main

import (
	"fmt"
	"time"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.ObjectPartInfo{
		{
			ETag:       "abc123",
			Number:     1,
			Size:       1024,
			ActualSize: 1000,
			ModTime:    time.Unix(1234567890, 0),
		},
		{
			ETag:       "def456",
			Number:     2,
			Size:       2048,
			ActualSize: 2000,
			ModTime:    time.Unix(1234567891, 0),
			Index:      []byte{1, 2, 3},
		},
		{
			ETag:       "ghi789",
			Number:     3,
			Size:       4096,
			ActualSize: 4000,
			ModTime:    time.Unix(1234567892, 0),
			Checksums:  map[string]string{"crc32": "abc123"},
		},
		{
			Number:     4,
			Size:       8192,
			ActualSize: 8000,
			ModTime:    time.Unix(1234567893, 0),
			Error:      "test error",
		},
		{
			ETag:       "jkl012",
			Number:     5,
			Size:       16384,
			ActualSize: 16000,
			ModTime:    time.Unix(1234567894, 0),
			Index:      []byte{4, 5, 6, 7},
			Checksums:  map[string]string{"sha256": "def456", "md5": "xyz"},
			Error:      "another error",
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		xl.PrintBytes(fmt.Sprintf("Case %d", i+1), data)
	}
}
