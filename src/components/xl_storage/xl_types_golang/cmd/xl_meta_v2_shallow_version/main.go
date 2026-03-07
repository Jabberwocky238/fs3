package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2ShallowVersion{
		{
			Header: xl.XlMetaV2VersionHeader{
				VersionID:   [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16},
				ModTime:     1234567890,
				Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
				VersionType: 1,
				Flags:       0,
				EcN:         4,
				EcM:         2,
			},
			Meta: []byte{0x01, 0x02, 0x03},
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2ShallowVersion Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
