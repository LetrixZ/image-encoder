#!/bin/bash

# Define the platforms and their targets
declare -A platforms
platforms["macos"]="x86_64-apple-darwin:x86_64-apple-darwin.libjxl.0.11.0.dylib:image-encoder.darwin-x64.node aarch64-apple-darwin:aarch64-apple-darwin.libjxl.0.11.0.dylib:image-encoder.darwin-arm64.node"
platforms["windows"]="x86_64-pc-windows-msvc:image-encoder.windows-x64-msvc.node aarch64-pc-windows-msvc:image-encoder.windows-x64-msvc.node"
platforms["linux"]="x86_64-unknown-linux-gnu:x86_64-unknown-linux-gnu.libjxl.so.0.11.0:image-encoder.linux-x64-gnu.node aarch64-unknown-linux-gnu:aarch64-unknown-linux-gnu.libjxl.so.0.11.0:image-encoder.linux-arm64-gnu.node x86_64-unknown-linux-musl:image-encoder.linux-x64-musl.node aarch64-unknown-linux-musl:image-encoder.linux-arm64-musl.node"

# Default to all platforms
buildPlatforms=("${!platforms[@]}")

# Parse command-line arguments
while getopts "p:" opt; do
  case $opt in
    p)
      IFS=',' read -ra userPlatforms <<< "$OPTARG"
      buildPlatforms=()
      for userPlatform in "${userPlatforms[@]}"; do
        userPlatform=$(echo "$userPlatform" | tr '[:upper:]' '[:lower:]' | xargs)
        if [[ -n ${platforms[$userPlatform]} ]]; then
          buildPlatforms+=("$userPlatform")
        fi
      done
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      exit 1
      ;;
  esac
done

# Build and rename binaries for the selected platforms
for platform in "${buildPlatforms[@]}"; do
  IFS=' ' read -ra targets <<< "${platforms[$platform]}"
  for target in "${targets[@]}"; do
    IFS=':' read -ra parts <<< "$target"
    target_name="${parts[0]}"
    target_lib="${parts[1]}"
    target_file="${parts[2]}"

    cmd=("bunx" "napi" "build" "--platform" "--target" "$target_name" "--release")

    echo "Running: ${cmd[@]}"
    DEP_JXL_LIB="libjxl/$target_lib" "${cmd[@]}"
    
    if [ $? -eq 0 ]; then
      mkdir -p bin
      mv "$target_file" "bin/$target_file" 2>/dev/null
    fi
  done
done
