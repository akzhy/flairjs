use lightningcss::{
  css_modules::{self},
  printer::PrinterOptions,
  stylesheet::{ParserOptions, StyleSheet, ToCssResult},
  targets::{Browsers, Features, Targets},
};

pub fn parse_css(css: &str) -> Result<ToCssResult, String> {
  let parser_options = ParserOptions {
    css_modules: Some(css_modules::Config {
      ..Default::default()
    }),
    ..Default::default()
  };

  let browsers = Browsers::from_browserslist(vec!["defaults"]).unwrap();
  let targets = Targets {
    browsers: browsers,
    include: Features::default() | Features::Nesting,
    ..Targets::default()
  };

  let stylesheet =
    StyleSheet::parse(css, parser_options).map_err(|e| format!("Failed to parse CSS: {}", e))?;

  let result = stylesheet.to_css(PrinterOptions {
    minify: false,
    targets: targets,
    ..Default::default()
  });

  let ret_value = match result {
    Ok(result) => result,
    Err(e) => return Err(format!("Failed to convert stylesheet to CSS: {}", e)),
  };
  Ok(ret_value)
}
