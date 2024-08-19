use derivative::Derivative;
use napi_derive::napi;
use rolldown_common::{GeneratedCodeOptions, GeneratedCodePreset};
use serde::Deserialize;

#[napi(object)]
#[derive(Deserialize, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BindingGeneratedCodeOptions {
  pub arrow_functions: Option<bool>,
  pub const_bindings: Option<bool>,
  pub object_shorthand: Option<bool>,
  #[napi(ts_type = "'es5' | 'es2015'")]
  pub preset: Option<String>,
  pub reserved_names_as_props: Option<bool>,
  pub symbols: Option<bool>,
}

impl Default for BindingGeneratedCodeOptions {
  fn default() -> Self {
    Self {
      arrow_functions: Some(false),
      const_bindings: Some(false),
      object_shorthand: Some(false),
      preset: Some("es2015".to_string()),
      reserved_names_as_props: Some(true),
      symbols: Some(false),
    }
  }
}

impl TryFrom<BindingGeneratedCodeOptions> for GeneratedCodeOptions {
  fn try_from(options: BindingGeneratedCodeOptions) -> anyhow::Result<Self> {
    let preset = options.preset.unwrap_or_default();
    Ok(Self {
      arrow_functions: options.arrow_functions.unwrap_or_default(),
      const_bindings: options.const_bindings.unwrap_or_default(),
      object_shorthand: options.object_shorthand.unwrap_or_default(),
      preset: match preset.as_str() {
        "es5" => GeneratedCodePreset::Es5,
        "es2015" => GeneratedCodePreset::Es2015,
        _ => return Err(anyhow::anyhow!("Invalid GeneratedCodePreset: {preset}")),
      },
      reserved_names_as_props: options.reserved_names_as_props.unwrap_or_default(),
      symbols: options.symbols.unwrap_or_default(),
    })
  }
  type Error = anyhow::Error;
}
