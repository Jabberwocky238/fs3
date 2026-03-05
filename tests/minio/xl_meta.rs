// xl.meta serialization/deserialization integration tests
// Tests compatibility with MinIO's xl.meta format using xlmeta-decoder.exe

use std::process::Command;
use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use s3_mount_gateway_rust::components::xl_storage::{
        XlMetaV2Version, VersionType, XlMetaV2Object, ErasureAlgo, ChecksumAlgo
    };

    fn run_decoder(args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let mut cmd = Command::new("./xlmeta-decoder.exe");
        cmd.args(args);

        if let Some(data) = input {
            use std::process::Stdio;
            use std::io::Write;
            cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| e.to_string())?;
            child.stdin.as_mut().unwrap().write_all(data).map_err(|e| e.to_string())?;
            let output = child.wait_with_output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }
            Ok(output.stdout)
        } else {
            let output = cmd.output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }
            Ok(output.stdout)
        }
    }

    #[test]
    fn test_version_roundtrip() {
        let json = r#"{
  "Type": 1,
  "V2Obj": {
    "ID": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
    "DDir": [17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
    "EcAlgo": 1,
    "EcM": 2,
    "EcN": 2,
    "EcBSize": 10485760,
    "EcIndex": 1,
    "EcDist": [1,2,3,4],
    "CSumAlgo": 1,
    "PartNums": [1],
    "PartETags": ["etag1"],
    "PartSizes": [1024],
    "PartASizes": [1024],
    "Size": 1024,
    "MTime": 1234567890,
    "MetaSys": {},
    "MetaUsr": {}
  }
}"#;

        let msgpack = run_decoder(&["encode-ver"], Some(json.as_bytes()))
            .expect("encode failed");
        let decoded = run_decoder(&["decode-ver"], Some(&msgpack))
            .expect("decode failed");

        let original: Value = serde_json::from_str(json).unwrap();
        let result: Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(original["Type"], result["Type"]);
    }

    #[test]
    fn test_delete_marker_roundtrip() {
        let json = r#"{
  "Type": 2,
  "DelObj": {
    "ID": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
    "MTime": 1234567890,
    "MetaSys": {}
  }
}"#;

        let msgpack = run_decoder(&["encode-ver"], Some(json.as_bytes()))
            .expect("encode failed");
        let decoded = run_decoder(&["decode-ver"], Some(&msgpack))
            .expect("decode failed");

        let original: Value = serde_json::from_str(json).unwrap();
        let result: Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(original["Type"], result["Type"]);
    }

    #[test]
    fn test_rust_decode_minio_encoded() {
        let json = r#"{
  "Type": 1,
  "V2Obj": {
    "ID": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
    "DDir": [17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
    "EcAlgo": 1,
    "EcM": 2,
    "EcN": 2,
    "EcBSize": 10485760,
    "EcIndex": 1,
    "EcDist": [1,2,3,4],
    "CSumAlgo": 1,
    "PartNums": [1],
    "PartETags": ["test"],
    "PartSizes": [1024],
    "PartASizes": [1024],
    "Size": 1024,
    "MTime": 1234567890,
    "MetaSys": {},
    "MetaUsr": {}
  }
}"#;

        let msgpack = run_decoder(&["encode-ver"], Some(json.as_bytes()))
            .expect("minio encode failed");

        let version = XlMetaV2Version::try_from(&msgpack[..])
            .expect("rust decode failed");

        assert_eq!(version.version_type, VersionType::Object);
    }

    #[test]
    fn test_minio_decode_rust_encoded() {
        use std::collections::HashMap;
        use s3_mount_gateway_rust::components::xl_storage::GoBytes;

        let obj = XlMetaV2Object {
            version_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
            data_dir: [17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 2,
            erasure_n: 2,
            erasure_block_size: 10485760,
            erasure_index: 1,
            erasure_dist: vec![1,2,3,4],
            checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1],
            part_etags: vec!["test".to_string()],
            part_sizes: vec![1024],
            part_actual_sizes: vec![1024],
            part_indices: vec![],
            size: 1024,
            mod_time: 1234567890,
            meta_sys: HashMap::new(),
            meta_user: HashMap::new(),
        };

        let version = XlMetaV2Version {
            version_type: VersionType::Object,
            object_v2: Some(obj),
            delete_marker: None,
        };

        let rust_encoded: GoBytes = (&version).into();
        let rust_encoded = rust_encoded.0;

        // MinIO decode
        let decoded = run_decoder(&["decode-ver"], Some(&rust_encoded))
            .expect("minio decode failed");

        let result: Value = serde_json::from_slice(&decoded).unwrap();

        // Verify all fields match
        assert_eq!(result["Type"], 1);
        assert_eq!(result["V2Obj"]["EcM"], 2);
        assert_eq!(result["V2Obj"]["EcN"], 2);
        assert_eq!(result["V2Obj"]["Size"], 1024);

        // Now encode with MinIO and compare binary
        let json = serde_json::to_string(&result).unwrap();
        let minio_encoded = run_decoder(&["encode-ver"], Some(json.as_bytes()))
            .expect("minio encode failed");

        // Binary should be identical
        if rust_encoded != minio_encoded {
            println!("Rust encoded ({} bytes):", rust_encoded.len());
            for (i, chunk) in rust_encoded.chunks(16).enumerate() {
                print!("{:04x}: ", i * 16);
                for b in chunk {
                    print!("{:02x} ", b);
                }
                println!();
            }
            println!("\nMinIO encoded ({} bytes):", minio_encoded.len());
            for (i, chunk) in minio_encoded.chunks(16).enumerate() {
                print!("{:04x}: ", i * 16);
                for b in chunk {
                    print!("{:02x} ", b);
                }
                println!();
            }
            panic!("Binary encoding mismatch!");
        }
    }
}
