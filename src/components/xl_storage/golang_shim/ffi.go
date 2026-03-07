package main

/*
#include <stdlib.h>
*/
import "C"
import (
	"unsafe"
)

//export EncodeXlMetaV2Object
func EncodeXlMetaV2Object(
	versionID *C.char,
	dataDir *C.char,
	erasureAlgo C.uchar,
	erasureM C.int,
	erasureN C.int,
	erasureBlockSize C.longlong,
	erasureIndex C.int,
	size C.longlong,
	modTime C.longlong,
	outBuf **C.char,
	outLen *C.int,
) C.int {
	obj := &xlMetaV2Object{
		ErasureAlgorithm: uint8(erasureAlgo),
		ErasureM:         int(erasureM),
		ErasureN:         int(erasureN),
		ErasureBlockSize: int64(erasureBlockSize),
		ErasureIndex:     int(erasureIndex),
		Size:             int64(size),
		ModTime:          int64(modTime),
	}

	// Copy version ID
	if versionID != nil {
		vidStr := C.GoString(versionID)
		copy(obj.VersionID[:], vidStr)
	}

	// Copy data dir
	if dataDir != nil {
		ddStr := C.GoString(dataDir)
		copy(obj.DataDir[:], ddStr)
	}

	// Marshal to msgpack
	data, err := obj.MarshalMsg(nil)
	if err != nil {
		return -1
	}

	// Allocate C memory
	*outLen = C.int(len(data))
	*outBuf = (*C.char)(C.CBytes(data))

	return 0
}

//export DecodeXlMetaV2Object
func DecodeXlMetaV2Object(
	inBuf *C.char,
	inLen C.int,
	versionID **C.char,
	size *C.longlong,
	modTime *C.longlong,
) C.int {
	data := C.GoBytes(unsafe.Pointer(inBuf), inLen)

	var obj xlMetaV2Object
	if _, err := obj.UnmarshalMsg(data); err != nil {
		return -1
	}

	*size = C.longlong(obj.Size)
	*modTime = C.longlong(obj.ModTime)
	*versionID = C.CString(string(obj.VersionID[:]))

	return 0
}

//export FreeCString
func FreeCString(s *C.char) {
	C.free(unsafe.Pointer(s))
}

func main() {}
