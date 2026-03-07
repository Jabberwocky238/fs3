package main

import (
	"fmt"
	"time"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV1Object{
		{
			Version: "1.0.0",
			Format:  "xl",
			Stat: xl.StatInfo{
				Size:    1024,
				ModTime: 1234567890,
				Name:    "test.txt",
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
			Minio: xl.MinioInfo{
				Release: "RELEASE.2024-01-01T00-00-00Z",
			},
		},
		{
			Version: "1.0.0",
			Format:  "xl",
			Stat: xl.StatInfo{
				Size:    2048,
				ModTime: 1234567891,
				Name:    "multipart.dat",
				Dir:     false,
				Mode:    0644,
			},
			Erasure: xl.ErasureInfo{
				Algorithm:    "reedsolomon",
				Data:         2,
				Parity:       1,
				BlockSize:    5242880,
				Index:        1,
				Distribution: []int{1, 2, 3},
			},
			Minio: xl.MinioInfo{
				Release: "RELEASE.2024-01-01T00-00-00Z",
			},
			Meta: map[string]string{"key": "value"},
			Parts: []xl.ObjectPartInfo{
				{
					ETag:       "abc123",
					Number:     1,
					Size:       1024,
					ActualSize: 1000,
					ModTime:    time.Unix(1234567890, 0),
				},
			},
			VersionID: "version-123",
			DataDir:   "data-dir-uuid",
		},
	}

	// Add more complex cases
	cases = append(cases, xl.XlMetaV1Object{
		Version: "1.0.0",
		Format:  "xl",
		Stat: xl.StatInfo{
			Size:    8192,
			ModTime: time.Unix(1234567892, 0),
			Name:    "large.bin",
			Dir:     false,
			Mode:    0600,
		},
		Erasure: xl.ErasureInfo{
			Algorithm:    "reedsolomon",
			Data:         8,
			Parity:       4,
			BlockSize:    20971520,
			Index:        3,
			Distribution: []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12},
			Checksum: []xl.ChecksumInfo{
				{PartNumber: 1, Algorithm: xl.SHA256, Hash: []byte{0xaa, 0xbb}},
				{PartNumber: 2, Algorithm: xl.BLAKE2b512, Hash: []byte{0xcc, 0xdd}},
			},
		},
		Minio: xl.MinioInfo{
			Release: "RELEASE.2024-02-01T00-00-00Z",
		},
		Meta: map[string]string{
			"content-type": "application/octet-stream",
			"x-amz-meta-custom": "value",
		},
		Parts: []xl.ObjectPartInfo{
			{ETag: "part1", Number: 1, Size: 4096, ActualSize: 4000, ModTime: time.Unix(1234567890, 0)},
			{ETag: "part2", Number: 2, Size: 4096, ActualSize: 4000, ModTime: time.Unix(1234567891, 0)},
		},
		VersionID: "v2",
		DataDir:   "dir2",
	})

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV1Object Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
