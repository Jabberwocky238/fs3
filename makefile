.PHONY: build clean run test

build:
	cargo build
	cp target/debug/s3_mount_gateway_rust.exe ./fs3.exe

build-minio:
	cd minio && go build -o minio.exe .
	cp minio/minio.exe ./minio.exe

clean:
	cargo clean
	rm -f ./fs3.exe
	rm -f ./minio.exe

run:
	cargo run

test:
	cargo test --test minio_tests
