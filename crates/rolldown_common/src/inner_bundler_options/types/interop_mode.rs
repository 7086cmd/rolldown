#[cfg(feature = "deserialize_bundler_options")]
use schemars::JsonSchema;
#[cfg(feature = "deserialize_bundler_options")]
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Default)]
#[cfg_attr(
  feature = "deserialize_bundler_options",
  derive(Deserialize, JsonSchema),
  serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub enum InteropMode {
  Compat,
  Auto,
  EsModule,
  #[default]
  Default,
  DefaultOnly,
}

impl From<String> for InteropMode {
  fn from(s: String) -> Self {
    match s.as_str() {
      "compat" => InteropMode::Compat,
      "auto" => InteropMode::Auto,
      "es-module" => InteropMode::EsModule,
      "default" => InteropMode::Default,
      "default-only" => InteropMode::DefaultOnly,
      _ => unreachable!("Invalid interop mode"),
    }
  }
}

impl Display for InteropMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let str = match self {
      InteropMode::Compat => "Compact".to_string(),
      InteropMode::Auto | InteropMode::EsModule => String::new(),
      InteropMode::Default => "Default".to_string(),
      InteropMode::DefaultOnly => "DefaultOnly".to_string(),
    };
    write!(f, "{str}")
  }
}

#[allow(unused)]
impl InteropMode {
  pub fn get_function_name(&self, default: bool) -> String {
    format!("_interop{}{}", if default { "Default" } else { "Namespace" }, self)
  }

  pub fn get_function(&self, is_default: bool, freeze: bool) -> Option<String> {
    let wrapper_head = format!("function {} (e) {{", self.get_function_name(is_default));
    match self {
      InteropMode::EsModule => None,
      InteropMode::Default | InteropMode::DefaultOnly if is_default => None,
      InteropMode::Compat if is_default => {
        Some(format!("{wrapper_head} return e && typeof e === 'object' && 'default' in e ? e : {{ default: e }}; }}"))
      }
      InteropMode::Auto if is_default => {
        Some(format!("{wrapper_head} return e && e.__esModule ? e : {{ default: e }}; }}"))
      }
      InteropMode::DefaultOnly if !is_default => {
        Some(format!("{wrapper_head} return Object.freeze({{ __proto__: null, default: e }}); }}"))
      }
      _ if !is_default => {
        let introduction = match self {
          InteropMode::Compat => "if (e && typeof e === 'object' && 'default' in e) return e;",
          InteropMode::Auto => "if (e && e.__esModule) return e;",
          InteropMode::Default => "",
          _ => unreachable!("Invalid interop mode"),
        };
        Some(format!("{wrapper_head} {{\
  {introduction}\
  if (e) {{\
    Object.keys(e).forEach(function (k) {{
	if (k !== 'default') {{
	  var d = Object.getOwnPropertyDescriptor(e, k);
		Object.defineProperty(n, k, d.get ? d : {{
		  enumerable: true,
	      get: function () {{ return e[k]; }}
		}});
	  }}
	}});
  }}
  n.default = e;
  return {}
}}", if freeze { "Object.freeze(n)" } else { "n" }))
      }
      _ => None
    }
  }
}
