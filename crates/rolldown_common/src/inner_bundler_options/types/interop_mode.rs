#[cfg(feature = "deserialize_bundler_options")]
use schemars::JsonSchema;
#[cfg(feature = "deserialize_bundler_options")]
use serde::Deserialize;

#[derive(Debug, Default)]
#[cfg_attr(
  feature = "deserialize_bundler_options",
  derive(Deserialize, JsonSchema),
  serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub enum InteropMode {
  Compact,
  Auto,
  EsModule,
  #[default]
  Default,
  DefaultOnly,
}

impl From<String> for InteropMode {
  fn from(s: String) -> Self {
    match s.as_str() {
      "compact" => InteropMode::Compact,
      "auto" => InteropMode::Auto,
      "esm" => InteropMode::EsModule,
      "default" => InteropMode::Default,
      "defaultOnly" => InteropMode::DefaultOnly,
      _ => InteropMode::Default,
    }
  }
}
