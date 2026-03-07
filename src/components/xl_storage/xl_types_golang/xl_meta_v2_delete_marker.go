package xl_types_golang

//go:generate msgp

type XlMetaV2DeleteMarker struct {
	VersionID [16]byte          `msg:"ID"`
	ModTime   int64             `msg:"MTime"`
	MetaSys   map[string][]byte `msg:"MetaSys,omitempty"`
}
