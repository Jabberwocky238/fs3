use serde_json::Value;
use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;
    use s3_mount_gateway_rust::components::xl_storage::XlStorage;
    use s3_mount_gateway_rust::types::errors::StorageError;
    use s3_mount_gateway_rust::types::s3::object_layer_types::Context;
    use s3_mount_gateway_rust::types::s3::storage_types::FileInfo;
    use s3_mount_gateway_rust::types::traits::storage_api::StorageMetadata;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    fn run_decoder(args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let mut cmd = Command::new("./xlmeta-decoder.exe");
        cmd.args(args);

        if let Some(data) = input {
            use std::io::Write;
            use std::process::Stdio;
            cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| e.to_string())?;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(data)
                .map_err(|e| e.to_string())?;
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

    fn decode_meta_file(path: &Path) -> Result<Vec<u8>, String> {
        let path_string = path.to_string_lossy().into_owned();
        run_decoder(&["decode-meta", &path_string], None)
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("fs3-{name}-{nanos}"))
    }

    fn test_context() -> Context {
        Context {
            request_id: "test-request".to_string(),
        }
    }

    fn test_file_info() -> FileInfo {
        FileInfo {
            volume: "bucket".to_string(),
            name: "object".to_string(),
            version_id: Uuid::new_v4().to_string(),
            size: 1024,
            data_dir: Uuid::new_v4().to_string(),
            etag: String::new(),
            content_type: "application/octet-stream".to_string(),
            user_metadata: Default::default(),
            erasure_index: 1,
            erasure_m: 1,
            erasure_n: 0,
        }
    }

    fn minio_object_json() -> &'static str {
        r#"{
  "Type": 1,
  "V2Obj": {
    "ID": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
    "DDir": [17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
    "EcAlgo": 1,
    "EcM": 1,
    "EcN": 0,
    "EcBSize": 1048576,
    "EcIndex": 1,
    "EcDist": [1],
    "CSumAlgo": 1,
    "PartNums": [1],
    "PartETags": [],
    "PartSizes": [1024],
    "PartASizes": [1024],
    "Size": 1024,
    "MTime": 1234567890,
    "MetaSys": {},
    "MetaUsr": {}
  }
}"#
    }

    #[test]
    fn test_version_roundtrip() {
        let json = minio_object_json();
        let msgpack = run_decoder(&["encode-ver"], Some(json.as_bytes())).expect("encode failed");
        let decoded = run_decoder(&["decode-ver"], Some(&msgpack)).expect("decode failed");

        let result: Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(result["Type"], 1);
        assert_eq!(result["V2Obj"]["Size"], 1024);
    }

    #[tokio::test]
    async fn test_rust_decode_minio_full_xl_meta() {
        let temp_dir = unique_temp_dir("decode-minio-meta");
        fs::create_dir_all(temp_dir.join("bucket").join("object")).unwrap();

        let minio_encoded = run_decoder(&["encode-meta"], Some(minio_object_json().as_bytes()))
            .expect("minio encode-meta failed");
        fs::write(
            temp_dir.join("bucket").join("object").join("xl.meta"),
            minio_encoded,
        )
        .unwrap();

        let storage = XlStorage::new(temp_dir.clone());
        let file_info = storage
            .read_version(&test_context(), "bucket", "object", "null")
            .await
            .map_err(|err: StorageError| err.to_string())
            .expect("fs3 read_version failed");

        assert_eq!(file_info.size, 1024);
        assert_eq!(file_info.volume, "bucket");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_minio_decode_rust_full_xl_meta() {
        let temp_dir = unique_temp_dir("decode-rust-meta");
        let storage = XlStorage::new(temp_dir.clone());
        let file_info = test_file_info();

        storage
            .write_metadata(&test_context(), "bucket", "object", file_info)
            .await
            .map_err(|err: StorageError| err.to_string())
            .expect("fs3 write_metadata failed");

        let xl_meta_path = temp_dir.join("bucket").join("object").join("xl.meta");
        let decoded = decode_meta_file(&xl_meta_path).expect("minio decode-meta failed");
        let result: Value = serde_json::from_slice(&decoded).unwrap();

        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["Type"], 1);
        assert_eq!(result[0]["V2Obj"]["Size"], 1024);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
