package main

import (
	"time"

	"github.com/dustin/go-humanize"
)

const (
	nullVersionID = "null"

	// Small file threshold below which data accompanies metadata from storage layer.
	smallFileThreshold = 128 * humanize.KiByte // Optimized for NVMe/SSDs

	// For hardrives it is possible to set this to a lower value to avoid any
	// spike in latency. But currently we are simply keeping it optimal for SSDs.

	// bigFileThreshold is the point where we add readahead to put operations.
	bigFileThreshold = 128 * humanize.MiByte

	// XL metadata file carries per object metadata.
	xlStorageFormatFile = "xl.meta"

	// XL metadata file backup file carries previous per object metadata.
	xlStorageFormatFileBackup = "xl.meta.bkp"

	globalMinioDefaultOwnerID      = "02d6176db174dc93cb1b899f7c6078f08654445fe8cf1b6ce98d8855f66bdbf4"
	globalMinioDefaultStorageClass = "STANDARD"
	globalWindowsOSName            = "windows"
	globalMacOSName                = "darwin"
	globalMinioModeFS              = "mode-server-fs"
	globalMinioModeErasureSD       = "mode-server-xl-single"
	globalMinioModeErasure         = "mode-server-xl"
	globalMinioModeDistErasure     = "mode-server-distributed-xl"
	globalDirSuffix                = "__XLDIR__"
)
const (
	// Maximum allowed form data field values. 64MiB is a guessed practical value
	// which is more than enough to accommodate any form data fields and headers.
	requestFormDataSize = 64 * humanize.MiByte

	// For any HTTP request, request body should be not more than 16GiB + requestFormDataSize
	// where, 16GiB is the maximum allowed object size for object upload.
	requestMaxBodySize = globalMaxObjectSize + requestFormDataSize

	// Maximum size for http headers - See: https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingMetadata.html
	maxHeaderSize = 8 * 1024

	// Maximum size for user-defined metadata - See: https://docs.aws.amazon.com/AmazonS3/latest/dev/UsingMetadata.html
	maxUserDataSize = 2 * 1024

	// maxBuckets upto 500000 for any MinIO deployment.
	maxBuckets = 500 * 1000
)

// ReservedMetadataPrefix is the prefix of a metadata key which
// is reserved and for internal use only.
const (
	ReservedMetadataPrefix      = "X-Minio-Internal-"
	ReservedMetadataPrefixLower = "x-minio-internal-"
)
const (
	// Maximum object size per PUT request is 5TB.
	// This is a divergence from S3 limit on purpose to support
	// use cases where users are going to upload large files
	// using 'curl' and presigned URL.
	globalMaxObjectSize = 5 * humanize.TiByte

	// Minimum Part size for multipart upload is 5MiB
	globalMinPartSize = 5 * humanize.MiByte

	// Maximum Part ID for multipart upload is 10000
	// (Acceptable values range from 1 to 10000 inclusive)
	globalMaxPartID = 10000
)

// Beginning of unix time is treated as sentinel value here.
var (
	timeSentinel     = time.Unix(0, 0).UTC()
	timeSentinel1970 = time.Unix(0, 1).UTC() // 1970 used for special cases when xlmeta.version == 0
)

const (
	throttleDeadline = 1 * time.Hour
	// ReplicationReset has reset id and timestamp of last reset operation
	ReplicationReset = "replication-reset"
	// ReplicationStatus has internal replication status - stringified representation of target's replication status for all replication
	// activity initiated from this cluster
	ReplicationStatus = "replication-status"
	// ReplicationTimestamp - the last time replication was initiated on this cluster for this object version
	ReplicationTimestamp = "replication-timestamp"
	// ReplicaStatus - this header is present if a replica was received by this cluster for this object version
	ReplicaStatus = "replica-status"
	// ReplicaTimestamp - the last time a replica was received by this cluster for this object version
	ReplicaTimestamp = "replica-timestamp"
	// TaggingTimestamp - the last time a tag metadata modification happened on this cluster for this object version
	TaggingTimestamp = "tagging-timestamp"
	// ObjectLockRetentionTimestamp - the last time a object lock metadata modification happened on this cluster for this object version
	ObjectLockRetentionTimestamp = "objectlock-retention-timestamp"
	// ObjectLockLegalHoldTimestamp - the last time a legal hold metadata modification happened on this cluster for this object version
	ObjectLockLegalHoldTimestamp = "objectlock-legalhold-timestamp"

	// ReplicationSsecChecksumHeader - the encrypted checksum of the SSE-C encrypted object.
	ReplicationSsecChecksumHeader = "X-Minio-Replication-Ssec-Crc"
)
