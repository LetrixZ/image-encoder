import { readFileSync } from "node:fs";

export function isMusl() {
  if (!process.report || typeof process.report.getReport !== "function") {
    try {
      const lddPath = require("child_process").execSync("which ldd").toString().trim();

      return readFileSync(lddPath, "utf8").includes("musl");
    } catch (e) {
      return true;
    }
  } else {
    //@ts-ignore
    const { glibcVersionRuntime } = process.report.getReport().header;

    return !glibcVersionRuntime;
  }
}
