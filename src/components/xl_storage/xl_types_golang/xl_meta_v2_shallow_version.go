package xl_types_golang

//go:generate msgp

type XlMetaV2ShallowVersion struct {
	Header XlMetaV2VersionHeader `msg:"h"`
	Meta   []byte                `msg:"m"`
}
