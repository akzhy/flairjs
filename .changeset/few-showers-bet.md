---
"@flairjs/core": minor
"@flairjs/parcel-transformer": minor
"@flairjs/rollup-plugin": minor
"@flairjs/bundler-shared": minor
"@flairjs/vite-plugin": minor
"@flairjs/webpack-loader": minor
---

- Add sourcemap support for CSS
- Add logs for unused CSS
- Fix some classnames not getting recognised
- \[Breaking] Updated TransformOutput to always have a value, with success property determining the success of the operation
- \[Breaking] Updated cssPreprocessor function to have an object as the first param instead of string
