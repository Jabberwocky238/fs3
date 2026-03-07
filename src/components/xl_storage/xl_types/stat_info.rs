use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatInfo {
    pub size: i64,
    #[serde(rename = "modTime")]
    pub mod_time: i64,
    pub name: String,
    pub dir: bool,
    pub mode: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msgpack_go_compat() {
        let cases = vec![
            (hex::decode("85a453697a65d10400a74d6f6454696d65c70c0500000000499602d200000000a44e616d65a8746573742e747874a3446972c2a44d6f6465cd01a4").unwrap(), 1024, "test.txt", false, 0o644),
            (hex::decode("85a453697a6500a74d6f6454696d65c70c0500000000499602d300000000a44e616d65a56d79646972a3446972c3a44d6f6465cd01ed").unwrap(), 0, "mydir", true, 0o755),
        ];

        for (i, (bytes, size, name, dir, mode)) in cases.into_iter().enumerate() {
            let obj: StatInfo = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(obj.size, size, "Case {} size", i + 1);
            assert_eq!(obj.name, name, "Case {} name", i + 1);
            assert_eq!(obj.dir, dir, "Case {} dir", i + 1);
            assert_eq!(obj.mode, mode, "Case {} mode", i + 1);
        }
    }
}
