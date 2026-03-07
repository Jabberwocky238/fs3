package xl_types_golang

//go:generate msgp

type VersionType uint8

const (
	InvalidVersionType VersionType = 0
	ObjectType         VersionType = 1
	DeleteType         VersionType = 2
	LegacyType         VersionType = 3
)

type XlMetaV2Version struct {
	Type             VersionType           `msg:"Type"`
	ObjectV1         []byte                `msg:"V1Obj,omitempty"`
	ObjectV2         *XlMetaV2Object       `msg:"V2Obj,omitempty"`
	DeleteMarker     *XlMetaV2DeleteMarker `msg:"DelObj,omitempty"`
	WrittenByVersion uint64                `msg:"v"`
}
