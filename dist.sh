#!/bin/bash
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc

bun run build --target aarch64-apple-darwin
bun run build --target x86_64-apple-darwin
bun run build --target aarch64-unknown-linux-gnu
bun run build --target aarch64-unknown-linux-musl
bun run build --target x86_64-unknown-linux-gnu
bun run build --target x86_64-unknown-linux-musl
