package xl_types_golang

//go:generate msgp

type VersionType uint8

const (
	InvalidVersionType VersionType = 0
	ObjectType         VersionType = 1
	DeleteType         VersionType = 2
	LegacyType         VersionType = 3
)

// xlMetaV2Version describes the journal entry, Type defines
// the current journal entry type other types might be nil based
// on what Type field carries, it is imperative for the caller
// to verify which journal type first before accessing rest of the fields.
type XlMetaV2Version struct {
	Type             VersionType           `json:"Type" msg:"Type"`
	ObjectV1         *XlMetaV1Object       `json:"V1Obj,omitempty" msg:"V1Obj,omitempty"`
	ObjectV2         *XlMetaV2Object       `json:"V2Obj,omitempty" msg:"V2Obj,omitempty"`
	DeleteMarker     *XlMetaV2DeleteMarker `json:"DelObj,omitempty" msg:"DelObj,omitempty"`
	WrittenByVersion uint64                `msg:"v"` // Tracks written by MinIO version
}
