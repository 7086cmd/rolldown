use crate::{
  types::generator::{GenerateContext, GenerateOutput, Generator},
  utils::{chunk::generate_rendered_chunk, render_ecma_module::render_ecma_module},
};

use super::format::{app::render_app, cjs::render_cjs, esm::render_esm};
use crate::ecmascript::format::utils::wrapper::render_wrapper;
use anyhow::Result;
use rolldown_common::{
  AssetMeta, EcmaAssetMeta, ModuleId, ModuleIdx, OutputFormat, PreliminaryAsset, RenderedModule,
};
use rolldown_error::DiagnosableResult;
use rolldown_plugin::HookAddonArgs;
use rolldown_sourcemap::Source;
use rolldown_utils::rayon::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use sugar_path::SugarPath;

pub type RenderedModuleSources = Vec<(ModuleIdx, ModuleId, Option<Vec<Box<dyn Source + Send>>>)>;

pub struct EcmaGenerator;

impl Generator for EcmaGenerator {
  #[allow(clippy::too_many_lines)]
  async fn render_preliminary_assets<'a>(
    ctx: &mut GenerateContext<'a>,
  ) -> Result<DiagnosableResult<GenerateOutput>> {
    let mut rendered_modules = FxHashMap::default();

    let rendered_module_sources = ctx
      .chunk
      .modules
      .par_iter()
      .copied()
      .filter_map(|id| ctx.link_output.module_table.modules[id].as_ecma())
      .map(|m| {
        (
          m.idx,
          m.id.clone(),
          render_ecma_module(
            m,
            &ctx.link_output.ast_table[m.ecma_ast_idx()].0,
            m.id.as_ref(),
            ctx.options,
          ),
        )
      })
      .collect::<Vec<_>>();

    rendered_module_sources.iter().for_each(|(_, module_id, _)| {
      // FIXME: NAPI-RS used CStr under the hood, so it can't handle null byte in the string.
      if !module_id.starts_with('\0') {
        rendered_modules.insert(module_id.clone(), RenderedModule { code: None });
      }
    });

    let rendered_chunk = generate_rendered_chunk(
      ctx.chunk,
      ctx.link_output,
      ctx.options,
      rendered_modules,
      ctx.chunk_graph,
    );

    let banner = {
      let injection = match ctx.options.banner.as_ref() {
        Some(hook) => hook.call(&rendered_chunk).await?,
        None => None,
      };
      ctx
        .plugin_driver
        .banner(HookAddonArgs { chunk: &rendered_chunk }, injection.unwrap_or_default())
        .await?
    };

    let intro = {
      let injection = match ctx.options.intro.as_ref() {
        Some(hook) => hook.call(&rendered_chunk).await?,
        None => None,
      };
      ctx
        .plugin_driver
        .intro(HookAddonArgs { chunk: &rendered_chunk }, injection.unwrap_or_default())
        .await?
    };

    let outro = {
      let injection = match ctx.options.outro.as_ref() {
        Some(hook) => hook.call(&rendered_chunk).await?,
        None => None,
      };
      ctx
        .plugin_driver
        .outro(HookAddonArgs { chunk: &rendered_chunk }, injection.unwrap_or_default())
        .await?
    };

    let footer = {
      let injection = match ctx.options.footer.as_ref() {
        Some(hook) => hook.call(&rendered_chunk).await?,
        None => None,
      };
      ctx
        .plugin_driver
        .footer(HookAddonArgs { chunk: &rendered_chunk }, injection.unwrap_or_default())
        .await?
    };

    let concat_source = match ctx.options.format {
      OutputFormat::Esm => render_esm(ctx, rendered_module_sources, banner, footer, intro, outro),
      OutputFormat::Cjs => {
        match render_cjs(ctx, rendered_module_sources, banner, footer, intro, outro) {
          Ok(concat_source) => concat_source,
          Err(errors) => return Ok(Err(errors)),
        }
      }
      OutputFormat::App => render_app(ctx, rendered_module_sources, banner, footer, intro, outro),
      OutputFormat::Iife | OutputFormat::Amd => {
        match render_wrapper(ctx, rendered_module_sources, banner, footer, intro, outro) {
          Ok(concat_source) => concat_source,
          Err(errors) => return Ok(Err(errors)),
        }
      }
    };

    let (content, mut map) = concat_source.content_and_sourcemap();

    // Here file path is generated by chunk file name template, it maybe including path segments.
    // So here need to read it's parent directory as file_dir.
    let file_path = ctx.options.cwd.as_path().join(&ctx.options.dir).join(
      ctx
        .chunk
        .preliminary_filename
        .as_deref()
        .expect("chunk file name should be generated before rendering")
        .as_str(),
    );
    let file_dir = file_path.parent().expect("chunk file name should have a parent");

    if let Some(map) = map.as_mut() {
      let paths =
        map.get_sources().map(|source| source.as_path().relative(file_dir)).collect::<Vec<_>>();
      // Here not normalize the windows path, the rollup `sourcemap_path_transform` ctx.options need to original path.
      let sources = paths.iter().map(|x| x.to_string_lossy()).collect::<Vec<_>>();
      map.set_sources(sources.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>());
    }

    Ok(Ok(GenerateOutput {
      assets: vec![PreliminaryAsset {
        origin_chunk: ctx.chunk_idx,
        content,
        map,
        meta: AssetMeta::from(EcmaAssetMeta { rendered_chunk }),
        augment_chunk_hash: None,
        file_dir: file_dir.to_path_buf(),
        preliminary_filename: ctx
          .chunk
          .preliminary_filename
          .clone()
          .expect("should have preliminary filename"),
      }],
      warnings: std::mem::take(&mut ctx.warnings),
    }))
  }
}
