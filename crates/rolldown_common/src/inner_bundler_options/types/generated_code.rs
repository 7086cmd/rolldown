#[cfg(feature = "deserialize_bundler_options")]
use schemars::JsonSchema;
#[cfg(feature = "deserialize_bundler_options")]
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Default)]
#[cfg_attr(
  feature = "deserialize_bundler_options",
  derive(Deserialize, JsonSchema),
  serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub enum GeneratedCodePreset {
  Es5,
  #[default]
  Es2015,
}

impl FromStr for GeneratedCodePreset {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "es5" => Ok(Self::Es5),
      "es2015" => Ok(Self::Es2015),
      _ => Err(format!("Invalid GeneratedCodePreset: {s}")),
    }
  }
}

#[derive(Debug)]
#[cfg_attr(
  feature = "deserialize_bundler_options",
  derive(Deserialize, JsonSchema),
  serde(rename_all = "camelCase", deny_unknown_fields)
)]
#[allow(clippy::struct_excessive_bools)]
pub struct GeneratedCodeOptions {
  pub arrow_functions: bool,
  pub const_bindings: bool,
  pub object_shorthand: bool,
  pub preset: GeneratedCodePreset,
  pub reserved_names_as_props: bool,
  pub symbols: bool,
}

impl Default for GeneratedCodeOptions {
  fn default() -> Self {
    Self {
      arrow_functions: false,
      const_bindings: false,
      object_shorthand: false,
      preset: GeneratedCodePreset::Es2015,
      reserved_names_as_props: true,
      symbols: false,
    }
  }
}