import * as esbuild from "esbuild";
import { existsSync } from "fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import path from "path";
import { pathToFileURL } from "url";
import { store } from "./store";

const __dirname = dirname(fileURLToPath(import.meta.url));

export interface GetUserThemeResult {
  theme: any;
  originalPath: string;
}

export const getUserTheme = async (options?: {
  ignoreCache?: boolean;
}): Promise<GetUserThemeResult | null> => {
  if (store.getUserTheme() && !options?.ignoreCache) {
    return store.getUserTheme();
  }
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
      store.setUserTheme({
        theme: userTheme.default,
        originalPath: userThemeFilePath,
      });
      return {
        theme: userTheme.default,
        originalPath: userThemeFilePath,
      };
    }

    store.setUserTheme(null);
    return null;
  } catch (error) {
    console.error("Error loading user theme:", error);
    return null;
  }
};
