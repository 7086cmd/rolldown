use napi_derive::napi;
use rolldown_common::AmdOptions;
use serde::Deserialize;

#[napi(object)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindingAmdOptions {
  pub id: Option<String>,
  pub define: Option<String>,
  pub auto_id: Option<bool>,
  pub base_path: Option<String>,
  pub force_js_extension_for_imports: Option<bool>,
}

impl From<BindingAmdOptions> for AmdOptions {
  fn from(value: BindingAmdOptions) -> Self {
    let auto_id = value.auto_id.unwrap_or(false);
    Self {
      id: value.id.unwrap_or(if auto_id { "main".to_string() } else { String::new() }),
      define: value.define.unwrap_or("define".to_string()),
      auto_id,
      base_path: value.base_path.unwrap_or_default(),
      force_js_extension_for_imports: value.force_js_extension_for_imports.unwrap_or(false),
    }
  }
}
