package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2Object{
		// Case 1: Minimal fields
		{
			VersionID: [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16},
			DataDir:   [16]byte{0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00},
			Size:      1024,
			ModTime:   1234567890,
		},
		// Case 2: Different IDs
		{
			VersionID: [16]byte{0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00},
			DataDir:   [16]byte{0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00},
			Size:      2048,
			ModTime:   1234567891,
		},
		// Case 3: Another minimal
		{
			VersionID: [16]byte{0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0},
			DataDir:   [16]byte{0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, 0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde},
			Size:      4096,
			ModTime:   1234567892,
		},
		// Case 4: With erasure + parts + metadata
		{
			VersionID:          [16]byte{0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88},
			DataDir:            [16]byte{0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12},
			ErasureAlgorithm:   1,
			ErasureM:           10,
			ErasureN:           5,
			ErasureBlockSize:   4194304,
			ErasureIndex:       3,
			ErasureDist:        []uint8{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14},
			BitrotChecksumAlgo: 1,
			PartNumbers:        []int{1, 2, 3},
			PartETags:          []string{"part1", "part2", "part3"},
			PartSizes:          []int64{1024, 2048, 4096},
			PartActualSizes:    []int64{1000, 2000, 4000},
			PartIndices:        [][]byte{{1, 2}, {3, 4}, {5, 6}},
			Size:               7168,
			ModTime:            1234567893,
			MetaSys:            map[string][]byte{"sys1": {0x01, 0x02}},
		},
		// Case 5: Large erasure dist, no parts
		{
			VersionID:          [16]byte{0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99},
			DataDir:            [16]byte{0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa},
			ErasureAlgorithm:   1,
			ErasureM:           12,
			ErasureN:           6,
			ErasureBlockSize:   8388608,
			ErasureIndex:       4,
			ErasureDist:        []uint8{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17},
			BitrotChecksumAlgo: 1,
			Size:               0,
			ModTime:            1234567894,
		},
		// Case 6: Multiple parts with user metadata
		{
			VersionID:          [16]byte{0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01},
			DataDir:            [16]byte{0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02},
			ErasureAlgorithm:   1,
			ErasureM:           2,
			ErasureN:           1,
			ErasureBlockSize:   262144,
			ErasureIndex:       0,
			ErasureDist:        []uint8{0, 1, 2},
			BitrotChecksumAlgo: 1,
			PartNumbers:        []int{1, 2, 3, 4, 5},
			PartETags:          []string{"e1", "e2", "e3", "e4", "e5"},
			PartSizes:          []int64{100, 200, 300, 400, 500},
			Size:               1500,
			ModTime:            1234567895,
			MetaUser:           map[string]string{"content-type": "application/octet-stream", "x-custom": "test"},
		},
		// Case 7: Large multipart with both metadata
		{
			VersionID:          [16]byte{0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0},
			DataDir:            [16]byte{0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, 0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde},
			ErasureAlgorithm:   1,
			ErasureM:           16,
			ErasureN:           8,
			ErasureBlockSize:   16777216,
			ErasureIndex:       7,
			ErasureDist:        []uint8{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23},
			BitrotChecksumAlgo: 1,
			PartNumbers:        []int{1, 2, 3, 4},
			PartETags:          []string{"large1", "large2", "large3", "large4"},
			PartSizes:          []int64{10485760, 10485760, 10485760, 10485760},
			PartActualSizes:    []int64{10000000, 10000000, 10000000, 10000000},
			PartIndices:        [][]byte{{0x01}, {0x02}, {0x03}, {0x04}},
			Size:               41943040,
			ModTime:            1234567896,
			MetaSys:            map[string][]byte{"encryption": {0xaa, 0xbb, 0xcc}, "compression": {0x01}},
			MetaUser:           map[string]string{"content-type": "video/mp4", "x-amz-meta-custom": "large-file"},
		},
		// Case 8: Many small parts
		{
			VersionID:          [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
			DataDir:            [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2},
			ErasureAlgorithm:   1,
			ErasureM:           1,
			ErasureN:           1,
			ErasureBlockSize:   65536,
			ErasureIndex:       0,
			ErasureDist:        []uint8{0, 1},
			BitrotChecksumAlgo: 1,
			PartNumbers:        []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10},
			PartETags:          []string{"p1", "p2", "p3", "p4", "p5", "p6", "p7", "p8", "p9", "p10"},
			PartSizes:          []int64{50, 50, 50, 50, 50, 50, 50, 50, 50, 50},
			PartActualSizes:    []int64{48, 48, 48, 48, 48, 48, 48, 48, 48, 48},
			PartIndices:        [][]byte{{0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}},
			Size:               500,
			ModTime:            1234567897,
			MetaSys:            map[string][]byte{"key": {0xff}},
			MetaUser:           map[string]string{"x-test": "multipart"},
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2Object Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
