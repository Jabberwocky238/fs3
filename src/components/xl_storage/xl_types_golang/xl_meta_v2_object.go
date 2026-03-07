package xl_types_golang

//go:generate msgp

type ErasureAlgo uint8

type ChecksumAlgo uint8

// List of currently supported erasure coding algorithms
const (
	invalidErasureAlgo ErasureAlgo = 0
	ReedSolomon        ErasureAlgo = 1
	lastErasureAlgo    ErasureAlgo = 2
)

// List of currently supported checksum algorithms
const (
	invalidChecksumAlgo ChecksumAlgo = 0
	HighwayHash         ChecksumAlgo = 1
	lastChecksumAlgo    ChecksumAlgo = 2
)

// XlMetaV2Object defines the data struct for object journal type
type XlMetaV2Object struct {
	VersionID          [16]byte          `json:"ID" msg:"ID"`                                    // Version ID
	DataDir            [16]byte          `json:"DDir" msg:"DDir"`                                // Data dir ID
	ErasureAlgorithm   ErasureAlgo       `json:"EcAlgo" msg:"EcAlgo"`                            // Erasure coding algorithm
	ErasureM           int               `json:"EcM" msg:"EcM"`                                  // Erasure data blocks
	ErasureN           int               `json:"EcN" msg:"EcN"`                                  // Erasure parity blocks
	ErasureBlockSize   int64             `json:"EcBSize" msg:"EcBSize"`                          // Erasure block size
	ErasureIndex       int               `json:"EcIndex" msg:"EcIndex"`                          // Erasure disk index
	ErasureDist        []uint8           `json:"EcDist" msg:"EcDist"`                            // Erasure distribution
	BitrotChecksumAlgo ChecksumAlgo      `json:"CSumAlgo" msg:"CSumAlgo"`                        // Bitrot checksum algo
	PartNumbers        []int             `json:"PartNums" msg:"PartNums"`                        // Part Numbers
	PartETags          []string          `json:"PartETags" msg:"PartETags,allownil"`             // Part ETags
	PartSizes          []int64           `json:"PartSizes" msg:"PartSizes"`                      // Part Sizes
	PartActualSizes    []int64           `json:"PartASizes,omitempty" msg:"PartASizes,allownil"` // Part ActualSizes (compression)
	PartIndices        [][]byte          `json:"PartIndices,omitempty" msg:"PartIdx,omitempty"`  // Part Indexes (compression)
	Size               int64             `json:"Size" msg:"Size"`                                // Object version size
	ModTime            int64             `json:"MTime" msg:"MTime"`                              // Object version modified time
	MetaSys            map[string][]byte `json:"MetaSys,omitempty" msg:"MetaSys,allownil"`       // Object version internal metadata
	MetaUser           map[string]string `json:"MetaUsr,omitempty" msg:"MetaUsr,allownil"`       // Object version metadata set by user
}
