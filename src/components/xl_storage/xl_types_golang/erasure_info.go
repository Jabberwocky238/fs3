package xl_types_golang

//go:generate msgp

type ErasureInfo struct {
	Algorithm    string         `json:"algorithm" msg:"algo"`
	Data         int            `json:"data" msg:"d"`
	Parity       int            `json:"parity" msg:"p"`
	BlockSize    int64          `json:"blockSize" msg:"bs"`
	Index        int            `json:"index" msg:"i"`
	Distribution []int          `json:"distribution" msg:"dist"`
	Checksum     []ChecksumInfo `json:"checksum,omitempty" msg:"cs,omitempty"`
}
