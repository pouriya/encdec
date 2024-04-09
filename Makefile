TARGET=$(shell rustc -vV | awk '$$1 == "host:"{print $$2}')
BUILD_DIR=${CURDIR}/build
VERSION=$(shell cat Cargo.toml | awk 'BEGIN{FS="[ \"]"}$$1 == "version"{print $$4;exit}')
CMD=${BUILD_DIR}/encdec-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}
DEV_CMD=${BUILD_DIR}/encdec-${VERSION}-${TARGET}-dev${RELEASE_FILENAME_POSTFIX}
RELEASE_FILENAME_POSTFIX=


all: release


release: ${BUILD_DIR}
	cargo build --release --target ${TARGET}
	@ cp ./target/${TARGET}/release/encdec ${CMD}
	@ ls -sh ${BUILD_DIR}/encdec-*


dev: ${BUILD_DIR}
	cargo build --target ${TARGET}
	@ cp ./target/${TARGET}/debug/encdec ${DEV_CMD}
	@ ls -sh ${BUILD_DIR}/encdec-*dev*


start-dev: dev
	${DEV_CMD}


lint:
	cargo fmt --verbose --check
	cargo check --target ${TARGET}
	cargo clippy --no-deps --target ${TARGET}


test:
	cargo test --target ${TARGET}


clean:
	cargo clean

dist-clean: clean
	rm -rf ${BUILD_DIR}


${BUILD_DIR}:
	@ mkdir -p ${BUILD_DIR}


.PHONY: all release deb dev start-dev lint test clean dist-clean
