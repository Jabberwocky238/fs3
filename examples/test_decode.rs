use s3_mount_gateway_rust::components::xl_storage::*;

fn main() {
    let data = std::fs::read("test_xl.meta").expect("read file");

    println!("File size: {}", data.len());
    println!("Header: {:?}", &data[0..8]);

    // Try decode
    match XlMetaV2::decode(&data) {
        Ok(meta) => {
            println!("✓ Success! {} versions", meta.versions.len());
        }
        Err(e) => {
            println!("✗ Error: {}", e);

            // Try manual decode to see where it fails
            if data.len() >= 8 && &data[0..4] == b"XL2 " {
                println!("✓ Header OK");
                let minor = u16::from_le_bytes([data[6], data[7]]);
                println!("✓ Version: 1.{}", minor);

                // Try to decode payload
                let pos = 9;
                if data[pos-1] == 0xc6 {
                    println!("✓ Found bin32 marker");
                    let payload_size = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    println!("✓ Payload size: {}", payload_size);
                }
            }
        }
    }
}
