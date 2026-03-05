// xl.meta serialization/deserialization integration tests
// Tests compatibility with MinIO's xl.meta format using xlmeta-decoder.exe

use std::process::Command;
use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use s3_mount_gateway_rust::components::xl_storage::{XlMetaV2Version, VersionType};

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

        let version = XlMetaV2Version::decode_from_gomap(&msgpack)
            .expect("rust decode failed");

        assert_eq!(version.version_type, VersionType::Object);
    }
}
