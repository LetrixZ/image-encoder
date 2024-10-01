const { $ } = require("bun");
const { readFileSync, renameSync } = require("fs");
const { join } = require("path");
const { platform, arch } = process;

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== "function") {
    try {
      const lddPath = require("child_process").execSync("which ldd").toString().trim();
      return readFileSync(lddPath, "utf8").includes("musl");
    } catch (e) {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

let target = undefined;
let extension = undefined;

switch (platform) {
  case "android":
    extension = "so";

    switch (arch) {
      case "arm64":
        target = "aarch64-linux-android";
        break;
      case "arm":
        target = "armv7-linux-androideabi";
        break;
      case "x64":
        target = "x86_64-linux-android";
        break;
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`);
    }

    break;
  case "win32":
    extension = "dll";

    switch (arch) {
      case "x64":
        target = "aarch64-pc-windows-msvc";
        break;
      case "arm64":
        target = "x86_64-pc-windows-msvc";
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
  case "freebsd":
    extension = "so";

    if (arch !== "x64") {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`);
    }

    target = "x86_64-unknown-freebsd";

    break;
  case "linux":
    extension = "so";

    switch (arch) {
      case "x64":
        if (isMusl()) {
          target = "x86_64-unknown-linux-musl";
        } else {
          target = "x86_64-unknown-linux-gnu";
        }
        break;
      case "arm64":
        if (isMusl()) {
          target = "aarch64-unknown-linux-musl";
        } else {
          target = "aarch64-unknown-linux-gnu";
        }
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
  renameSync(join("target", target, "release", `libimage_encoder.${extension}`), join("bin", `${platform}-${arch}.node`));
}
