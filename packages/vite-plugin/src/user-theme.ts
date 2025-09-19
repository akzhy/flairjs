import path from "path";
import { pathToFileURL } from "url";

export const getUserTheme = async () : Promise<{
  theme: any;
  path: string
} | null> => {
  try {
    const userThemeFilePath = path.resolve(process.cwd(), "flair.theme.ts");
    
    const cacheBuster = Date.now();
    const fileUrl = pathToFileURL(userThemeFilePath).href;
    const userTheme = await import(`${fileUrl}?update=${cacheBuster}`);

    if (userTheme.default) {
      return {
        theme: userTheme.default,
        path: userThemeFilePath
      };
    }

    return null;
  } catch (error) {
    console.error("Error loading user theme:", error);
    return null;
  }
}