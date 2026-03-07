package xl_types_golang

//go:generate msgp

type BitrotAlgorithm uint32

const (
	SHA256           BitrotAlgorithm = 1
	HighwayHash256   BitrotAlgorithm = 2
	HighwayHash256S  BitrotAlgorithm = 3
	BLAKE2b512       BitrotAlgorithm = 4
)

type ChecksumInfo struct {
	PartNumber int             `json:"partNumber" msg:"pn"`
	Algorithm  BitrotAlgorithm `json:"algorithm" msg:"a"`
	Hash       []byte          `json:"hash" msg:"h"`
}
