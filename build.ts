import { mkdirSync, renameSync } from "fs";
import { join } from "path";
import { isMusl } from "./utils";

const { platform, arch } = process;

let target: string | undefined = undefined;
let extension: string | undefined = undefined;

switch (platform) {
  case "win32":
    extension = "dll";

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
    extension = "dylib";

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
    extension = "so";

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
    mkdirSync("bin");
  } catch {}

  renameSync(join("target", target, "release", `libimage_encoder.${extension}`), join("bin", `${platform}-${arch}.node`));
}
