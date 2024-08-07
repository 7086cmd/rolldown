use crate::ecmascript::format::AddRawString;
use crate::utils::chunk::namespace_marker::render_namespace_markers;
use crate::utils::chunk::render_interop::render_interop_chunk_imports;
use crate::{
  ecmascript::ecma_generator::RenderedModuleSources,
  types::generator::GenerateContext,
  utils::chunk::{
    determine_export_mode::determine_export_mode,
    determine_use_strict::determine_use_strict,
    render_chunk_exports::{get_export_items, render_chunk_exports},
  },
};
use arcstr::ArcStr;
use rolldown_common::{ChunkKind, OutputExports};
use rolldown_error::{BuildDiagnostic, DiagnosableResult};
use rolldown_sourcemap::{ConcatSource, RawSource};
use rolldown_utils::ecma_script::legitimize_identifier_name;

// TODO refactor it to `wrap.rs` to reuse it for other formats (e.g. amd, umd).
pub fn render_iife(
  ctx: &mut GenerateContext<'_>,
  module_sources: RenderedModuleSources,
  banner: Option<String>,
  footer: Option<String>,
  intro: Option<String>,
  outro: Option<String>,
  invoke: bool,
) -> DiagnosableResult<ConcatSource> {
  let mut concat_source = ConcatSource::default();

  concat_source.add_optional_raw(banner);

  // iife wrapper start
  let export_items = get_export_items(ctx.chunk, ctx.link_output);
  let has_exports = !export_items.is_empty();
  let has_default_export = export_items.iter().any(|(name, _)| name.as_str() == "default");
  let entry_module = match ctx.chunk.kind {
    ChunkKind::EntryPoint { module, .. } => {
      &ctx.link_output.module_table.modules[module].as_ecma().expect("should be ecma module")
    }
    ChunkKind::Common => unreachable!("iife should be entry point chunk"),
  };
  let export_mode = determine_export_mode(ctx, entry_module, &export_items)?;
  let named_exports = matches!(&export_mode, OutputExports::Named);

  let (import_code, externals) = render_interop_chunk_imports(ctx);

  let (input_args, output_args) =
    render_iife_arguments(ctx, &externals, has_exports && named_exports);

  concat_source.add_source(Box::new(RawSource::new(format!(
    "{}(function({}) {{\n",
    if let Some(name) = &ctx.options.name {
      format!("var {name} = ")
    } else {
      ctx
        .warnings
        .push(BuildDiagnostic::missing_name_option_for_iife_export().with_severity_warning());
      String::new()
    },
    input_args
  ))));

  if determine_use_strict(ctx) {
    concat_source.add_raw("\"use strict\";".to_string());
  }

  concat_source.add_optional_raw(intro);

  if named_exports {
    let marker = render_namespace_markers(&ctx.options.es_module, has_default_export, false);
    concat_source.add_optional_raw(marker);
  }

  concat_source.add_raw(import_code);

  // chunk content
  // TODO indent chunk content for iife format
  module_sources.into_iter().for_each(|(_, _, module_render_output)| {
    if let Some(emitted_sources) = module_render_output {
      for source in emitted_sources {
        concat_source.add_source(source);
      }
    }
  });

  // iife exports
  let exports = render_chunk_exports(ctx, Some(&export_mode));
  concat_source.add_optional_raw(exports);

  concat_source.add_optional_raw(outro);

  if named_exports && has_exports {
    // We need to add `return exports;` here only if using `named`, because the default value is returned when using `default` in `render_chunk_exports`.
    concat_source.add_raw("return exports;".to_string());
  }

  // iife wrapper end
  if invoke {
    concat_source.add_raw(format!("}})({output_args});"));
  } else {
    concat_source.add_raw("})".to_string());
  }

  concat_source.add_optional_raw(footer);

  Ok(concat_source)
}

fn render_iife_arguments(
  ctx: &mut GenerateContext<'_>,
  externals: &[String],
  exports_key: bool,
) -> (String, String) {
  let mut input_args = if exports_key { vec!["exports".to_string()] } else { vec![] };
  let mut output_args = if exports_key { vec!["{}".to_string()] } else { vec![] };
  let globals = &ctx.options.globals;
  externals.iter().for_each(|external| {
    input_args.push(legitimize_identifier_name(external).to_string());
    if let Some(global) = globals.get(external) {
      output_args.push(legitimize_identifier_name(global).to_string());
    } else {
      let target = legitimize_identifier_name(external).to_string();
      ctx.warnings.push(
        BuildDiagnostic::missing_global_name(ArcStr::from(external), ArcStr::from(&target))
          .with_severity_warning(),
      );
      output_args.push(target.to_string());
    }
  });
  (input_args.join(", "), output_args.join(", "))
}
