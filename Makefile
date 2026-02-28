# ── fs3 Makefile ─────────────────────────────────────────────────────
#
# 三种构建模式:
#   minimal  — 空 feature，最小核心
#   default  — policy + multi-user
#   full     — 全部 feature (含所有存储后端)
#
# 用法:
#   make                  # 默认构建三种模式
#   make build-minimal    # 空 feature
#   make build-default    # default feature
#   make build-full       # 全部 feature
#   make run              # default 模式运行
#   make test             # 运行测试
#   make clean            # 清理

SUFFIX  := .exe
ifneq ($(OS),Windows_NT)
  SUFFIX :=
endif

# ── features ────────────────────────────────────────────────────────

FEAT_DEFAULT := policy,multi-user
FEAT_FULL    := policy,multi-user,storage-sqlite,storage-postgres,storage-k8sconfigmap

# ── 目录 ────────────────────────────────────────────────────────────

SRC_BIN := target/debug/s3_mount_gateway_rust$(SUFFIX)
SRC_REL := target/release/s3_mount_gateway_rust$(SUFFIX)

# ── build ───────────────────────────────────────────────────────────

.PHONY: build build-minimal build-default build-full
.PHONY: release release-minimal release-default release-full

build: build-minimal build-default build-full

build-minimal:
	cargo build --no-default-features
	cp $(SRC_BIN) ./fs3-minimal$(SUFFIX)

build-default:
	cargo build --no-default-features --features $(FEAT_DEFAULT)
	cp $(SRC_BIN) ./fs3$(SUFFIX)

build-full:
	cargo build --no-default-features --features $(FEAT_FULL)
	cp $(SRC_BIN) ./fs3-full$(SUFFIX)

release: release-minimal release-default release-full

release-minimal:
	cargo build --release --no-default-features
	cp $(SRC_REL) ./fs3-minimal$(SUFFIX)

release-default:
	cargo build --release --no-default-features --features $(FEAT_DEFAULT)
	cp $(SRC_REL) ./fs3$(SUFFIX)

release-full:
	cargo build --release --no-default-features --features $(FEAT_FULL)
	cp $(SRC_REL) ./fs3-full$(SUFFIX)

# ── run ─────────────────────────────────────────────────────────────

run: 
	cargo run

run-minimal: 
	cargo run --no-default-features

run-full: 
	cargo run --no-default-features --features $(FEAT_FULL)

# ── test / check ────────────────────────────────────────────────────

.PHONY: check check-minimal check-default check-full
.PHONY: test test-minimal test-default test-full clippy fmt

check: check-minimal check-default check-full

check-minimal:
	cargo check --no-default-features

check-default:
	cargo check --no-default-features --features $(FEAT_DEFAULT)

check-full:
	cargo check --no-default-features --features $(FEAT_FULL)

test: test-minimal test-default test-full

test-minimal:
	cargo test --no-default-features

test-default:
	cargo test --no-default-features --features $(FEAT_DEFAULT)

test-full:
	cargo test --no-default-features --features $(FEAT_FULL)

clippy:
	cargo clippy --no-default-features --features $(FEAT_FULL) -- -D warnings

fmt:
	cargo fmt

# ── clean ───────────────────────────────────────────────────────────

.PHONY: clean distclean

clean:
	cargo clean
	rm fs3$(SUFFIX) fs3-minimal$(SUFFIX) fs3-full$(SUFFIX) 
	rm libfs3k8sconfigmap.dll libfs3k8sconfigmap.so libfs3k8sconfigmap.dylib

distclean: clean
	rm -f fs3$(SUFFIX) fs3-minimal$(SUFFIX) fs3-full$(SUFFIX)
