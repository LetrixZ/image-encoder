import { readFileSync, renameSync } from "fs";

const { platform, arch } = process;

const baseURL = "https://git.letrix.xyz/fermin/image-encoder/releases/download/0.0.1";

let target: string | undefined = undefined;
let extension: string | undefined = undefined;
let filename: string | undefined = undefined;

function isMusl() {
  if (!process.report || typeof process.report.getReport !== "function") {
    try {
      const lddPath = require("child_process").execSync("which ldd").toString().trim();
      return readFileSync(lddPath, "utf8").includes("musl");
    } catch (e) {
      return true;
    }
  } else {
    // @ts-ignore
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

const supportedArchitectures = ["arm64", "x64"];

switch (platform) {
  case "android":
    if (!supportedArchitectures.includes(arch)) {
      throw new Error(`Unsupported architecture on Android ${arch}`);
    }

    filename = `android-${arch}.node`;

    break;
  case "win32":
    if (!supportedArchitectures.includes(arch)) {
      throw new Error(`Unsupported architecture on Android ${arch}`);
    }

    filename = `win32-${arch}-msvc.node`;

    break;
  case "darwin":
    if (!supportedArchitectures.includes(arch)) {
      throw new Error(`Unsupported architecture on Android ${arch}`);
    }

    filename = `darwin-${arch}.node`;

    break;
  case "freebsd":
    if (arch !== "x64") {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`);
    }

    filename = `freebsd-${arch}.node`;

    break;
  case "linux":
    if (!supportedArchitectures.includes(arch)) {
      throw new Error(`Unsupported architecture on Android ${arch}`);
    }

    filename = `linux-${arch}-${isMusl() ? "musl" : "gnu"}.node`;

    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

const downloadUrl = `${baseURL}/${platform}-${arch}.node`;

console.info(`Downloading binary from ${downloadUrl}`);

const res = await fetch(downloadUrl);

if (res.status !== 200) {
  throw new Error(`Failed to download binary from '${downloadUrl}'`);
}

await Bun.write("index." + filename, res);
