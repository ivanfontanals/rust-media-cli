.DEFAULT_GOAL := local-all

SHELL := /bin/bash

RUST_VERSION ?= 1.56.0 # $(shell head -n 1 rust-toolchain)
TEST_RUST_LOG ?= "debug"

KERNEL := $(shell uname -s)
ifeq ($(KERNEL),Linux)
	OS := linux
endif
ifeq ($(KERNEL),Darwin)
	OS := macos
endif


PROJECT_DIR := $(realpath $(CURDIR))
TARGET_DIR := $(PROJECT_DIR)/target
BINARY_NAME := car-model
BINARY_DIR := $(TARGET_DIR)/release/$(BINARY_NAME)

DOCKER_IMAGE := $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG)
DOCKERFILE := Dockerfile

KUBECTL_NAMESPACE = $(shell kubectl config view --minify --output 'jsonpath={..namespace}')

# --[ Setup ]---------------------------------------------------------------------

rust-toolchain-install:
	which "rustup" > /dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	rustup toolchain list && rustc --version && cargo --version
	[ `rustup toolchain list | grep $(RUST_VERSION) | wc -l` -gt "0" ] || rustup toolchain install $(RUST_VERSION)

rust-toolchain-setup-macos: rust-toolchain-install
	rustup default $(RUST_VERSION)
	rustup override set $(RUST_VERSION)
	rustup toolchain list && rustc --version && cargo --version

rust-toolchain-setup-linux: rust-toolchain-install
	rustup default $(RUST_VERSION)-x86_64-unknown-linux-gnu
	rustup override set $(RUST_VERSION)-x86_64-unknown-linux-gnu
	rustup toolchain list && rustc --version && cargo --version

rust-toolchain-setup: rust-toolchain-setup-$(OS)

rust-cargo-update:
	rustup component remove cargo
	rustup component add cargo
	rustup component remove clippy
	rustup component add clippy

# --[ Build ]---------------------------------------------------------------------

local-all: format clippy build test

build:
	cargo build $(BUILD_PROFILE_ARG) --all-targets

format:
	cargo fmt --all

check-format:
	cargo fmt --all -- --check

clippy:
	cargo clippy $(BUILD_PROFILE_ARG) --all-targets -- -D warnings

test:
	RUST_LOG=$(TEST_RUST_LOG) cargo test $(TEST_PROFILE_ARG) --bins -- --nocapture

test-all: .env
	RUST_LOG=$(TEST_RUST_LOG) cargo test $(TEST_PROFILE_ARG) --all-targets -- --nocapture

doc:
	cargo doc --open
	
run: .env
	cargo run

analyze: .env
	cargo run analyze  -f /Users/ivan.fontanals/My_Projects/photos_test/small/ -t images

phash: .env
	cargo run analyze  -f /Users/ivan.fontanals/My_Projects/photos_test/small/phash/ -t images -v	

media-info:
	cargo run info -f /Users/ivan.fontanals/My_Projects/photos_test/

clean:
	cargo clean

.env:
	touch .env

# --[ Docker ]---------------------------------------------------------------------

docker-binary-build-linux: build

docker-binary-build-macos:
	@echo "Cross-compiling application ..."
	$(DOCKER) run -ti --rm \
		-w /usr/src \
		-v "$(PROJECT_DIR):/usr/src/:delegated" \
		-v "${HOME}/.cargo/registry:/usr/local/cargo/registry:delegated" \
		rust:$(RUST_VERSION) \
		cargo build $(BUILD_PROFILE_ARG)

docker-binary-copy: docker-binary-build-$(OS)
	@echo "Preparing binary for Docker image ..."
	cp "$(BINARY_DIR)" "$(PROJECT_DIR)"

docker-build: docker-binary-copy
	@echo "Building Docker image ..."
	$(DOCKER) build $(DOCKER_BUILD_ARGS) -t "$(DOCKER_IMAGE)" -f "$(DOCKERFILE)" "$(PROJECT_DIR)"

docker-login:
	docker login $(ARTIFACTORY_DOCKER_REGISTRY)

docker-push: docker-build
	@echo "Pushing Docker image ..."
	$(DOCKER) push "$(DOCKER_IMAGE)"

docker-run: docker-build
	@echo "Running Docker image ..."
	$(DOCKER) run -ti --rm \
		-v "${HOME}/.kube:/root/.kube:ro" \
		-v "${HOME}/.aws:/root/.aws:ro" \
		"$(DOCKER_IMAGE)"

docker-clean:
	@echo "Cleaning Docker temporary files ..."
	rm -f $(PROJECT_DIR)/$(BINARY)

# --[ CI ]---------------------------------------------------------------------

ci-test: ci-update-project-version
	@echo "Running all the tests ..."
	$(MAKE) check-format clippy test-all

ci-release: ci-update-project-version
	@echo "Releasing Docker image ..."
	$(MAKE) docker-push $(if $(IS_TAG),ci-release-unicron-store,)

ci-release-unicron-store: docker-push
	@echo "Deploying to Unicron Store ..."
	script/custom/publish_chart.sh

ci-update-project-version:
	@echo "Configuring application version to $(PROJECT_VERSION) ..."
	sed -i'' -E "s/^version = \".*\"/version = \"$(PROJECT_VERSION)\"/g" Cargo.toml

# -----------------------------------------------------------------------------


.PHONY: build clippy format check-format test run clean local-all
.PHONY: docker-binary-build-linux docker-binary-build-macos docker-binary-copy docker-build docker-login docker-push docker-run docker-clean
.PHONY: ci-test ci-release ci-release-unicron-store ci-update-project-version ci-build
