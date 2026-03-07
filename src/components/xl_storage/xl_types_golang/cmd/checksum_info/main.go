package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.ChecksumInfo{
		{
			PartNumber: 1,
			Algorithm:  xl.SHA256,
			Hash:       []byte{0x01, 0x02, 0x03},
		},
		{
			PartNumber: 2,
			Algorithm:  xl.HighwayHash256,
			Hash:       []byte{0xaa, 0xbb, 0xcc, 0xdd},
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// ChecksumInfo Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
