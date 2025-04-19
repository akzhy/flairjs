import nodePath from "path";
import fs from "fs";
import { createHash } from "crypto";
const CACHE_DIR_NAME = ".cache";

export const createCacheDir = () => {
  const relDirectory = import.meta?.dirname;

  const cacheDir = nodePath.join(relDirectory, CACHE_DIR_NAME);

  if (!fs.existsSync(cacheDir)) {
    fs.mkdirSync(cacheDir, { recursive: true });
  }

  return cacheDir;
};

export const createCacheCSSFile = ({
  id,
  css,
}: {
  id: string;
  css: string;
}) => {
  const cacheDir = createCacheDir();
  const pathHash = createHash("md5").update(id).digest("hex");
  const cacheFilePath = nodePath.join(cacheDir, `${pathHash}.css`);
  fs.writeFileSync(cacheFilePath, css);
  return {
    path: cacheFilePath,
    name: `${pathHash}.css`,
  };
};
