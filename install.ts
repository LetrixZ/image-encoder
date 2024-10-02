import { version } from "./package.json";

const { platform, arch } = process;

const baseURL = `https://git.letrix.xyz/fermin/image-encoder/releases/download/${version}`;

let binary: string | undefined = `${platform}-${arch}.node`;

switch (platform) {
  case "win32":
    binary = `${platform}-${arch}-msvc.node`;
    break;
  case "linux":
    binary = `${platform}-${arch}-gnu.node`;
    break;
}

const downloadUrl = `${baseURL}/${platform}-${arch}.node`;

console.info(`Downloading binary from: ${downloadUrl}`);

const res = await fetch(downloadUrl);

if (res.status !== 200) {
  throw new Error(`Failed to download binary for ${platform} ${arch}`);
}

await Bun.write("index." + binary, res);
