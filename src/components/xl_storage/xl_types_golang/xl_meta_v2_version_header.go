package xl_types_golang

//go:generate msgp

type XlMetaV2VersionHeader struct {
	VersionID   [16]byte `msg:"vid"`
	ModTime     int64    `msg:"mt"`
	Signature   [4]byte  `msg:"sig"`
	VersionType uint8    `msg:"vt"`
	Flags       uint8    `msg:"f"`
	EcN         uint8    `msg:"n"`
	EcM         uint8    `msg:"m"`
}
