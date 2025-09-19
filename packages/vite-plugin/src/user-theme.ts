import path from "path";

export const getUserTheme = () => {
  try {
    const userThemeFilePath = path.resolve(process.cwd(), "flair.theme.ts");
    const userTheme = require(userThemeFilePath);

    if (userTheme.default) {
      return userTheme.default;
    }

    return null;
  } catch (error) {
    return null;
  }
}