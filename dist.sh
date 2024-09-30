#!/bin/bash
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc

yarn build --target x86_64-apple-darwin
yarn build --target x86_64-unknown-linux-gnu
yarn build --target x86_64-unknown-linux-musl
yarn build --target aarch64-apple-darwin
yarn build --target aarch64-unknown-linux-gnu
yarn build --target aarch64-unknown-linux-musl