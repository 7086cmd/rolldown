use crate::utils::{fetch_module, get_extension, valid_and_parse};
use dashmap::DashMap;
use rolldown_common::ModuleType;
use rolldown_plugin::{
  HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
  HookResolveIdReturn, Plugin, SharedPluginContext,
};
use rustc_hash::FxBuildHasher;
use std::borrow::Cow;
use std::future::Future;
use crate::transformer::transform_source;

#[derive(Debug)]
pub struct ResolvedHttpUrl {
  pub module_type: ModuleType,
  pub data: String,
}

#[derive(Debug, Default)]
pub struct HttpResolvePlugin {
  resolved_http_url: DashMap<String, ResolvedHttpUrl, FxBuildHasher>,
}

impl Plugin for HttpResolvePlugin {
  fn name(&self) -> Cow<'static, str> {
    "rolldown:http-resolve".into()
  }

  fn resolve_id(
    &self,
    _ctx: &SharedPluginContext,
    args: &HookResolveIdArgs<'_>,
  ) -> impl Future<Output = HookResolveIdReturn> + Send {
    async {
      let Some(url) = valid_and_parse(args.specifier) else {
        return Ok(None);
      };
      let body = fetch_module(&url).await?;
      let body = transform_source(&body, url.as_str())?;
      let module_type = ModuleType::from_known_str(get_extension(&url).unwrap_or(""))?;
      self
        .resolved_http_url
        .insert(args.specifier.to_string(), ResolvedHttpUrl { module_type, data: body });

      Ok(Some(HookResolveIdOutput { id: args.specifier.to_string(), ..Default::default() }))
    }
  }

  fn load(
    &self,
    _ctx: &SharedPluginContext,
    args: &HookLoadArgs<'_>,
  ) -> impl Future<Output = HookLoadReturn> + Send {
    async {
      let Some(_) = valid_and_parse(args.id) else {
        return Ok(None);
      };
      let Some(resolved) = self.resolved_http_url.get(args.id) else {
        return Ok(None);
      };

      let module_type = resolved.module_type.clone();

      Ok(Some(HookLoadOutput {
        code: resolved.data.clone(),
        module_type: Some(module_type),
        ..Default::default()
      }))
    }
  }
}
