RUSTC ?= rustc
CARGO ?= cargo

TARGET ?=
PROFILE ?= release

ifeq ($(PROFILE),release)
  CARGO_PROFILE := --release
  ARTIFACT_DIR := target/$(TARGET)/release
else
  CARGO_PROFILE :=
  ARTIFACT_DIR := target/$(TARGET)/debug
endif

LIBNAME := libffmpeg_ffi.a

.PHONY: all clean

all:
	$(CARGO) build $(CARGO_PROFILE) $(if $(TARGET),--target $(TARGET),)
	@cp "$(ARTIFACT_DIR)/$(LIBNAME)" .

clean:
	$(CARGO) clean
	@rm -f "$(LIBNAME)"

