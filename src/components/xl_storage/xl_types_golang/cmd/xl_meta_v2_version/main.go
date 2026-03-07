package main

import (
	"fmt"

	xl "github.com/fs3/xl_types_golang"
)

func main() {
	cases := []xl.XlMetaV2Version{
		{
			Type:             xl.ObjectType,
			WrittenByVersion: 1,
		},
		{
			Type:             xl.DeleteType,
			WrittenByVersion: 2,
		},
		{
			Type:             xl.LegacyType,
			ObjectV1:         []byte{0x01, 0x02, 0x03},
			WrittenByVersion: 3,
		},
	}

	for i, obj := range cases {
		data, _ := obj.MarshalMsg(nil)
		fmt.Printf("// XlMetaV2Version Case %d\n", i+1)
		fmt.Printf("\"%s\"\n\n", xl.StringifyBytes(data))
	}
}
