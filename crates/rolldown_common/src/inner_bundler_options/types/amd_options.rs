use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug)]
#[cfg_attr(
  feature = "deserialize_bundler_options",
  derive(Deserialize, JsonSchema),
  serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct AmdOptions {
  pub id: String,
  pub define: String,
  pub auto_id: bool,
  pub base_path: String,
  pub force_js_extension_for_imports: bool,
}

impl Default for AmdOptions {
  fn default() -> Self {
    Self {
      id: String::new(),
      define: "define".to_string(),
      auto_id: false,
      base_path: String::new(),
      force_js_extension_for_imports: false,
    }
  }
}
