package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.ErasureInfo{
		{
			Algorithm:    "reedsolomon",
			Data:         4,
			Parity:       2,
			BlockSize:    10485760,
			Index:        1,
			Distribution: []int{1, 2, 3, 4, 5, 6},
		},
		{
			Algorithm:    "reedsolomon",
			Data:         2,
			Parity:       1,
			BlockSize:    5242880,
			Index:        2,
			Distribution: []int{1, 2, 3},
			Checksum: []xl.ChecksumInfo{
				{PartNumber: 1, Algorithm: xl.SHA256, Hash: []byte{0x01}},
			},
		},
	}

	cases = append(cases, xl.ErasureInfo{
		Algorithm:    "reedsolomon",
		Data:         8,
		Parity:       4,
		BlockSize:    20971520,
		Index:        5,
		Distribution: []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12},
		Checksum: []xl.ChecksumInfo{
			{PartNumber: 1, Algorithm: xl.SHA256, Hash: []byte{0x01, 0x02}},
			{PartNumber: 2, Algorithm: xl.HighwayHash256S, Hash: []byte{0x03, 0x04}},
			{PartNumber: 3, Algorithm: xl.BLAKE2b512, Hash: []byte{0x05, 0x06}},
		},
	})

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// ErasureInfo Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
