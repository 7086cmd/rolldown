use derivative::Derivative;
use serde::Deserialize;

use super::binding_hook_side_effects::BindingHookSideEffects;
use crate::types::binding_sourcemap::BindingSourcemap;

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BindingHookTransformOutput {
  pub code: Option<String>,
  pub side_effects: Option<BindingHookSideEffects>,
  pub map: Option<BindingSourcemap>,
}

impl TryFrom<BindingHookTransformOutput> for rolldown_plugin::HookTransformOutput {
  type Error = anyhow::Error;

  fn try_from(value: BindingHookTransformOutput) -> Result<Self, Self::Error> {
    Ok(rolldown_plugin::HookTransformOutput {
      code: value.code,
      map: value.map.map(TryInto::try_into).transpose()?,
      side_effects: value.side_effects.map(Into::into),
    })
  }
}
