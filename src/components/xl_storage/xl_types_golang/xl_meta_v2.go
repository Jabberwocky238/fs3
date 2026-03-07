package xl_types_golang

// xlMetaInlineData is serialized data in [string][]byte pairs.
type XlMetaInlineData []byte

// xlMetaInlineDataVer indicates the version of the inline data structure.
const XlMetaInlineDataVer = 1

//go:generate msgp

type XlMetaV2 struct {
	Versions []XlMetaV2ShallowVersion

	// data will contain raw data if any.
	// data will be one or more versions indexed by versionID.
	// To remove all data set to nil.
	Data XlMetaInlineData

	// metadata version.
	MetaV uint8
}
