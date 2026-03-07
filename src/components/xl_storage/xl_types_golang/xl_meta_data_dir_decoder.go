package xl_types_golang

//go:generate msgp

type XlMetaDataDirDecoder struct {
	ObjectV2 *struct {
		DataDir [16]byte `msg:"DDir"`
	} `msg:"V2Obj,omitempty"`
}
