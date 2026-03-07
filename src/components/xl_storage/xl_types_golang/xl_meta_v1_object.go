package xl_types_golang

//go:generate msgp

// A xlMetaV1Object represents `xl.meta` metadata header.
type XlMetaV1Object struct {
	Version string   `json:"version"` // Version of the current `xl.meta`.
	Format  string   `json:"format"`  // Format of the current `xl.meta`.
	Stat    StatInfo `json:"stat"`    // Stat of the current object `xl.meta`.
	// Erasure coded info for the current object `xl.meta`.
	Erasure ErasureInfo `json:"erasure"`
	// MinIO release tag for current object `xl.meta`.
	Minio struct {
		Release string `json:"release"`
	} `json:"minio"`
	// Metadata map for current object `xl.meta`.
	Meta map[string]string `json:"meta,omitempty"`
	// Captures all the individual object `xl.meta`.
	Parts []ObjectPartInfo `json:"parts,omitempty"`

	// Dummy values used for legacy use cases.
	VersionID string `json:"versionId,omitempty"`
	DataDir   string `json:"dataDir,omitempty"` // always points to "legacy"
}
