package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2VersionHeader{
		{
			VersionID:   [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16},
			ModTime:     1234567890,
			Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
			VersionType: 1,
			Flags:       0,
			EcN:         4,
			EcM:         2,
		},
		{
			VersionID:   [16]byte{0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00},
			ModTime:     1234567891,
			Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
			VersionType: 2,
			Flags:       3,
			EcN:         2,
			EcM:         1,
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2VersionHeader Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
