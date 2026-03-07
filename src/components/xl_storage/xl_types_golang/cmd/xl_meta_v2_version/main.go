package main

import (
	"fmt"
	"time"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2Version{
		// Case 1: ObjectType with V2
		{
			Type: xl.ObjectType,
			ObjectV2: &xl.XlMetaV2Object{
				VersionID: [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16},
				DataDir:   [16]byte{0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00},
				Size:      1024,
				ModTime:   1234567890,
			},
			WrittenByVersion: 1,
		},
		// Case 2: DeleteType
		{
			Type: xl.DeleteType,
			DeleteMarker: &xl.XlMetaV2DeleteMarker{
				VersionID: [16]byte{0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00},
				ModTime:   1234567891,
			},
			WrittenByVersion: 2,
		},
		// Case 3: LegacyType with V1
		{
			Type: xl.LegacyType,
			ObjectV1: &xl.XlMetaV1Object{
				Version: "1.0.0",
				Format:  "xl",
				Stat: xl.StatInfo{
					Size:    2048,
					ModTime: time.Unix(1234567892, 0),
					Name:    "legacy.dat",
					Dir:     false,
					Mode:    0644,
				},
				Erasure: xl.ErasureInfo{
					Algorithm:    "reedsolomon",
					Data:         4,
					Parity:       2,
					BlockSize:    10485760,
					Index:        1,
					Distribution: []int{1, 2, 3, 4, 5, 6},
				},
			},
			WrittenByVersion: 3,
		},
		// Case 4: ObjectType with erasure
		{
			Type: xl.ObjectType,
			ObjectV2: &xl.XlMetaV2Object{
				VersionID:          [16]byte{0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88},
				DataDir:            [16]byte{0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12},
				ErasureAlgorithm:   1,
				ErasureM:           8,
				ErasureN:           4,
				ErasureBlockSize:   4194304,
				ErasureIndex:       2,
				ErasureDist:        []uint8{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11},
				BitrotChecksumAlgo: 1,
				Size:               4096,
				ModTime:            1234567893,
			},
			WrittenByVersion: 4,
		},
		// Case 5: DeleteType with metadata
		{
			Type: xl.DeleteType,
			DeleteMarker: &xl.XlMetaV2DeleteMarker{
				VersionID: [16]byte{0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0},
				ModTime:   1234567894,
				MetaSys:   map[string][]byte{"key1": {0x01, 0x02}},
			},
			WrittenByVersion: 5,
		},
		// Case 6: ObjectType with parts
		{
			Type: xl.ObjectType,
			ObjectV2: &xl.XlMetaV2Object{
				VersionID:       [16]byte{0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01},
				DataDir:         [16]byte{0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02},
				PartNumbers:     []int{1, 2, 3},
				PartETags:       []string{"e1", "e2", "e3"},
				PartSizes:       []int64{100, 200, 300},
				PartActualSizes: []int64{90, 190, 290},
				Size:            600,
				ModTime:         1234567895,
			},
			WrittenByVersion: 6,
		},
		// Case 7: LegacyType with parts
		{
			Type: xl.LegacyType,
			ObjectV1: &xl.XlMetaV1Object{
				Version: "1.0.0",
				Format:  "xl",
				Stat: xl.StatInfo{
					Size:    8192,
					ModTime: time.Unix(1234567896, 0),
					Name:    "multipart.dat",
					Dir:     false,
					Mode:    0600,
				},
				Erasure: xl.ErasureInfo{
					Algorithm:    "reedsolomon",
					Data:         2,
					Parity:       1,
					BlockSize:    5242880,
					Index:        0,
					Distribution: []int{1, 2, 3},
				},
				Parts: []xl.ObjectPartInfo{
					{ETag: "part1", Number: 1, Size: 4096, ActualSize: 4000, ModTime: time.Unix(1234567890, 0)},
					{ETag: "part2", Number: 2, Size: 4096, ActualSize: 4000, ModTime: time.Unix(1234567891, 0)},
				},
				VersionID: "v1",
			},
			WrittenByVersion: 7,
		},
		// Case 8: ObjectType with metadata
		{
			Type: xl.ObjectType,
			ObjectV2: &xl.XlMetaV2Object{
				VersionID: [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
				DataDir:   [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2},
				Size:      500,
				ModTime:   1234567897,
				MetaSys:   map[string][]byte{"key": {0xff}},
				MetaUser:  map[string]string{"x-test": "value"},
			},
			WrittenByVersion: 8,
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2Version Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
