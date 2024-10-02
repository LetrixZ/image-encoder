import { mkdir, rename } from "node:fs/promises";
import { join } from "node:path";
import { isMusl } from "./utils";

const { platform, arch } = process;

let target: string | undefined = undefined;
let output: string | undefined = undefined;

switch (platform) {
  case "win32":
    output = "image_encoder.dll";

    switch (arch) {
      case "x64":
        target = "x86_64-pc-windows-msvc";
        break;
      case "arm64":
        target = "aarch64-pc-windows-msvc";
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }

    break;
  case "darwin":
    output = "libimage_encoder.dylib";

    switch (arch) {
      case "x64":
        target = "x86_64-apple-darwin";
        break;
      case "arm64":
        target = "aarch64-apple-darwin";
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }

    break;
  case "linux":
    output = "libimage_encoder.so";

    if (isMusl()) {
      throw new Error("Musl is Unsupported");
    }

    switch (arch) {
      case "x64":
        target = "x86_64-unknown-linux-gnu";
        break;
      case "arm64":
        target = "aarch64-unknown-linux-gnu";
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }

    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

const subprocess = Bun.spawn({
  cmd: ["cargo", "build", "--release", "--target", target],
});

await subprocess.exited;

if (!subprocess.exitCode) {
  try {
    await mkdir("bin");
  } catch {}

  rename(join("target", target, "release", output), join("bin", `${platform}-${arch}.node`));
}
