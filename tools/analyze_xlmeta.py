#!/usr/bin/env python
"""分析MinIO xl.meta并生成Rust msgpack编码代码"""
import sys

def analyze_xlmeta(filepath):
    with open(filepath, 'rb') as f:
        data = f.read()

    # 跳过XL2头部
    if data[:4] != b'XL2 ':
        print("Not a valid xl.meta file")
        return

    # 解析payload
    payload_len = int.from_bytes(data[9:13], 'big')
    payload = data[13:13+payload_len]

    print(f"Payload length: {payload_len}")
    print(f"Payload hex:")

    # 输出payload的hex，每16字节一行
    for i in range(0, len(payload), 16):
        chunk = payload[i:i+16]
        hex_str = ' '.join(f'{b:02x}' for b in chunk)
        print(f"  {i:04x}: {hex_str}")

    print(f"\nCRC: {int.from_bytes(data[13+payload_len+1:13+payload_len+5], 'big'):08x}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python analyze_xlmeta.py <xl.meta>")
        sys.exit(1)

    analyze_xlmeta(sys.argv[1])
