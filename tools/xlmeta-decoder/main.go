package main

import (
	"fmt"
	"io"
	"os"

	"github.com/minio/minio/cmd"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage:")
		fmt.Println("  xlmeta-decoder decode-meta <xl.meta>           # Decode full xl.meta")
		fmt.Println("  xlmeta-decoder encode-meta < input.json        # Encode full xl.meta")
		fmt.Println("  xlmeta-decoder decode-obj < msgpack            # Decode xlMetaV2Object")
		fmt.Println("  xlmeta-decoder encode-obj < json               # Encode xlMetaV2Object")
		fmt.Println("  xlmeta-decoder decode-ver < msgpack            # Decode xlMetaV2Version")
		fmt.Println("  xlmeta-decoder encode-ver < json               # Encode xlMetaV2Version")
		fmt.Println("  xlmeta-decoder decode-hdr < msgpack            # Decode xlMetaV2VersionHeader")
		fmt.Println("  xlmeta-decoder encode-hdr < json               # Encode xlMetaV2VersionHeader")
		fmt.Println("  xlmeta-decoder decode-dm < msgpack             # Decode xlMetaV2DeleteMarker")
		fmt.Println("  xlmeta-decoder encode-dm < json                # Encode xlMetaV2DeleteMarker")
		os.Exit(1)
	}

	command := os.Args[1]

	switch command {
	case "decode-meta":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "Usage: xlmeta-decoder decode-meta <xl.meta>")
			os.Exit(1)
		}
		decodeMeta(os.Args[2])
	case "encode-meta":
		encodeMeta()
	case "decode-obj":
		decodeStruct("obj")
	case "encode-obj":
		encodeStruct("obj")
	case "decode-ver":
		decodeStruct("ver")
	case "encode-ver":
		encodeStruct("ver")
	case "decode-hdr":
		decodeStruct("hdr")
	case "encode-hdr":
		encodeStruct("hdr")
	case "decode-dm":
		decodeStruct("dm")
	case "encode-dm":
		encodeStruct("dm")
	default:
		decodeMeta(os.Args[1])
	}
}

func decodeMeta(file string) {
	data, err := os.ReadFile(file)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	jsonData, err := cmd.DecodeXLMeta(data)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	os.Stdout.Write(jsonData)
	fmt.Println()
}

func encodeMeta() {
	jsonData, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	data, err := cmd.EncodeXLMetaFromJSON(jsonData)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	os.Stdout.Write(data)
}

func decodeStruct(typ string) {
	data, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	jsonData, err := cmd.DecodeStruct(typ, data)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	os.Stdout.Write(jsonData)
	fmt.Println()
}

func encodeStruct(typ string) {
	jsonData, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	data, err := cmd.EncodeStruct(typ, jsonData)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	os.Stdout.Write(data)
}
