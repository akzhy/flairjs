import * as esbuild from "esbuild";
import { existsSync } from "fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import path from "path";
import { pathToFileURL } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export const getUserTheme = async (): Promise<{
  theme: any;
  originalPath: string;
} | null> => {
  try {
    let userThemeFilePath = path.resolve(process.cwd(), "flair.theme.ts");
    if (!existsSync(userThemeFilePath)) {
      userThemeFilePath = path.resolve(process.cwd(), "flair.theme.js");
    }
    if (!existsSync(userThemeFilePath)) {
      return null;
    }

    const outFile = path.resolve(__dirname, `flair.theme.js`);

    await esbuild.build({
      entryPoints: [userThemeFilePath],
      outfile: outFile,
      platform: "node",
      bundle: true,
      format: "esm",
      external: ["*"],
    });

    const cacheBuster = Date.now();
    const fileUrl = pathToFileURL(outFile).href;
    const userTheme = await import(`${fileUrl}?update=${cacheBuster}`);

    if (userTheme.default) {
      return {
        theme: userTheme.default,
        originalPath: userThemeFilePath,
      };
    }

    return null;
  } catch (error) {
    console.error("Error loading user theme:", error);
    return null;
  }
};
