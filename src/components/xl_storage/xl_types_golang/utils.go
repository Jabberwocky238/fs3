package xl_types_golang

import (
	"fmt"
)

func PrintBytes(name string, data []byte) {
	fmt.Printf("// %s\n", name)
	fmt.Printf("vec![\n    ")
	for i, b := range data {
		if i > 0 && i%12 == 0 {
			fmt.Printf("\n    ")
		}
		fmt.Printf("0x%02x, ", b)
	}
	fmt.Printf("\n],\n\n")
}

func StringifyBytes(data []byte) string {
	hexStr := ""
	for _, b := range data {
		hexStr += fmt.Sprintf("%02x", b)
	}
	return hexStr
}
