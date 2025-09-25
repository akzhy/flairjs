class Store {
  private fileNameToGeneratedCssNameMap = new Map<string, string>();
  private lastThemeUpdate: number = Date.now();
  private userTheme: Map<number, { theme: any; originalPath: string } | null> =
    new Map();

  getUserTheme() {
    return this.userTheme.get(this.lastThemeUpdate) ?? null;
  }

  setUserTheme(theme: { theme: any; originalPath: string } | null) {
    this.userTheme.set(this.lastThemeUpdate, theme);
  }

  getLastThemeUpdate(): number {
    return this.lastThemeUpdate;
  }

  setLastThemeUpdate(timestamp: number) {
    this.lastThemeUpdate = timestamp;
  }

  setFileNameToGeneratedCssNameMap(fileName: string, generatedCssName: string) {
    this.fileNameToGeneratedCssNameMap.set(fileName, generatedCssName);
  }

  getGeneratedCssName(fileName: string): string | undefined {
    return this.fileNameToGeneratedCssNameMap.get(fileName);
  }
}

export const store = new Store();
