package main

import (
	"fmt"
	"time"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.StatInfo{
		{
			Size:    1024,
			ModTime: time.Unix(1234567890, 0),
			Name:    "test.txt",
			Dir:     false,
			Mode:    0644,
		},
		{
			Size:    0,
			ModTime: time.Unix(1234567891, 0),
			Name:    "mydir",
			Dir:     true,
			Mode:    0755,
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// StatInfo Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
