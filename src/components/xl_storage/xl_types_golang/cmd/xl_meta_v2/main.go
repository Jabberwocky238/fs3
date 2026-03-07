package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2{
		// Case 1: Empty
		{
			Versions: []xl.XlMetaV2ShallowVersion{},
			Data:     xl.XlMetaInlineData{},
			MetaV:    1,
		},
		// Case 2: Single version
		{
			Versions: []xl.XlMetaV2ShallowVersion{
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
			},
			Data:  xl.XlMetaInlineData{0xaa, 0xbb},
			MetaV: 1,
		},
		// Case 3: Two versions
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00},
						ModTime:     1234567891,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 1,
						Flags:       1,
						EcN:         8,
						EcM:         4,
					},
					Meta: []byte{0x11, 0x22},
				},
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0},
						ModTime:     1234567892,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 2,
						Flags:       2,
						EcN:         2,
						EcM:         1,
					},
					Meta: []byte{0x33, 0x44, 0x55},
				},
			},
			Data:  xl.XlMetaInlineData{0xcc, 0xdd, 0xee},
			MetaV: 1,
		},
		// Case 4: Three versions
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88},
						ModTime:     1234567893,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 1,
						Flags:       3,
						EcN:         16,
						EcM:         8,
					},
					Meta: []byte{0xa1, 0xa2},
				},
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12},
						ModTime:     1234567894,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 1,
						Flags:       0,
						EcN:         6,
						EcM:         3,
					},
					Meta: []byte{0xb1, 0xb2, 0xb3},
				},
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99},
						ModTime:     1234567895,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 2,
						Flags:       1,
						EcN:         12,
						EcM:         6,
					},
					Meta: []byte{0xc1},
				},
			},
			Data:  xl.XlMetaInlineData{0x01, 0x02, 0x03, 0x04},
			MetaV: 1,
		},
		// Case 5: Large data
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01},
						ModTime:     1234567896,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 1,
						Flags:       0,
						EcN:         1,
						EcM:         1,
					},
					Meta: []byte{0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88},
				},
			},
			Data:  xl.XlMetaInlineData{0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0},
			MetaV: 1,
		},
		// Case 6: Four versions
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}, ModTime: 1234567897, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 0, EcN: 2, EcM: 1}, Meta: []byte{0x01}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2}, ModTime: 1234567898, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 1, EcN: 4, EcM: 2}, Meta: []byte{0x02}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3}, ModTime: 1234567899, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 2, Flags: 0, EcN: 8, EcM: 4}, Meta: []byte{0x03}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4}, ModTime: 1234567900, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 2, EcN: 16, EcM: 8}, Meta: []byte{0x04}},
			},
			Data:  xl.XlMetaInlineData{0xf0, 0xf1, 0xf2},
			MetaV: 1,
		},
		// Case 7: Five versions
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff}, ModTime: 1234567901, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 0, EcN: 1, EcM: 1}, Meta: []byte{0xa0}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef}, ModTime: 1234567902, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 1, EcN: 2, EcM: 1}, Meta: []byte{0xb0}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf}, ModTime: 1234567903, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 2, Flags: 0, EcN: 4, EcM: 2}, Meta: []byte{0xc0}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf}, ModTime: 1234567904, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 2, EcN: 8, EcM: 4}, Meta: []byte{0xd0}},
				{Header: xl.XlMetaV2VersionHeader{VersionID: [16]byte{0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf}, ModTime: 1234567905, Signature: [4]byte{0x78, 0x6c, 0x32, 0x20}, VersionType: 1, Flags: 3, EcN: 16, EcM: 8}, Meta: []byte{0xe0}},
			},
			Data:  xl.XlMetaInlineData{0x11, 0x22, 0x33, 0x44, 0x55},
			MetaV: 1,
		},
		// Case 8: MetaV=2
		{
			Versions: []xl.XlMetaV2ShallowVersion{
				{
					Header: xl.XlMetaV2VersionHeader{
						VersionID:   [16]byte{0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf},
						ModTime:     1234567906,
						Signature:   [4]byte{0x78, 0x6c, 0x32, 0x20},
						VersionType: 1,
						Flags:       0,
						EcN:         10,
						EcM:         5,
					},
					Meta: []byte{0x99, 0x88, 0x77},
				},
			},
			Data:  xl.XlMetaInlineData{0xde, 0xad, 0xbe, 0xef},
			MetaV: 2,
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2 Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
