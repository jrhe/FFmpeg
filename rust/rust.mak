# Rust integration (experimental)

RUSTC ?= rustc
CARGO ?= cargo

RUST_TARGET ?=

RUST_PROFILE ?= release

ifeq ($(RUST_PROFILE),release)
  RUST_CARGO_PROFILE := --release
  RUST_ARTIFACT_SUBDIR := release
else
  RUST_CARGO_PROFILE :=
  RUST_ARTIFACT_SUBDIR := debug
endif

RUST_CARGO_TARGET_ARG := $(if $(RUST_TARGET),--target $(RUST_TARGET),)

RUST_FFMPEG_FFI_DIR := $(SRC_PATH)/rust/ffmpeg-ffi
RUST_FFMPEG_FFI_LIB := $(RUST_FFMPEG_FFI_DIR)/target/$(if $(RUST_TARGET),$(RUST_TARGET),)/$(RUST_ARTIFACT_SUBDIR)/libffmpeg_ffi.a

RUST_FFMPEG_FFI_LDFLAGS := $(RUST_FFMPEG_FFI_LIB) -ldl -lpthread

$(RUST_FFMPEG_FFI_LIB):
	$(M)cd $(RUST_FFMPEG_FFI_DIR) && $(CARGO) -q build $(RUST_CARGO_PROFILE) $(RUST_CARGO_TARGET_ARG)

clean::
	$(Q)$(CARGO) -q -C $(RUST_FFMPEG_FFI_DIR) clean || true
