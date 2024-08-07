use crate::ecmascript::format::AddRawString;
use crate::utils::chunk::determine_export_mode::determine_export_mode;
use crate::utils::chunk::namespace_marker::render_namespace_markers;
use crate::utils::chunk::render_chunk_exports::get_export_items;
use crate::utils::chunk::render_interop::render_interop_chunk_imports;
use crate::{
  ecmascript::ecma_generator::RenderedModuleSources,
  types::generator::GenerateContext,
  utils::chunk::{
    determine_use_strict::determine_use_strict, render_chunk_exports::render_chunk_exports,
  },
};
use rolldown_common::{ChunkKind, ExportsKind, Module, OutputExports, WrapKind};
use rolldown_error::DiagnosableResult;
use rolldown_sourcemap::{ConcatSource, RawSource};

pub fn render_cjs(
  ctx: &mut GenerateContext<'_>,
  module_sources: RenderedModuleSources,
  banner: Option<String>,
  footer: Option<String>,
  intro: Option<String>,
  outro: Option<String>,
) -> DiagnosableResult<ConcatSource> {
  let mut concat_source = ConcatSource::default();

  concat_source.add_optional_raw(banner);

  if determine_use_strict(ctx) {
    concat_source.add_source(Box::new(RawSource::new("\"use strict\";".to_string())));
  }

  concat_source.add_optional_raw(intro);

  // Note that the determined `export_mode` should be used in `render_chunk_exports` to render exports.
  // We also need to get the export mode for rendering the namespace markers.
  // So we determine the export mode (from auto) here and use it in the following code.
  let export_mode = if let ChunkKind::EntryPoint { module: entry_id, .. } = ctx.chunk.kind {
    if let Module::Ecma(entry_module) = &ctx.link_output.module_table.modules[entry_id] {
      if matches!(entry_module.exports_kind, ExportsKind::Esm) {
        let export_items = get_export_items(ctx.chunk, ctx.link_output);
        let has_default_export = export_items.iter().any(|(name, _)| name.as_str() == "default");
        let export_mode = determine_export_mode(ctx, entry_module, &export_items)?;
        // Only `named` export can we render the namespace markers.
        if matches!(&export_mode, OutputExports::Named) {
          let marker = render_namespace_markers(&ctx.options.es_module, has_default_export, false);
          concat_source.add_optional_raw(marker);
        }
        let meta = &ctx.link_output.metas[entry_id];
        meta.require_bindings_for_star_exports.iter().for_each(|(importee_idx, binding_ref)| {
          let importee = &ctx.link_output.module_table.modules[*importee_idx];
          let binding_ref_name =
            ctx.link_output.symbols.canonical_name_for(*binding_ref, &ctx.chunk.canonical_names);
            let import_stmt = "Object.keys($NAME).forEach(function (k) {
  if (k !== 'default' && !Object.prototype.hasOwnProperty.call(exports, k)) Object.defineProperty(exports, k, {
    enumerable: true,
    get: function () { return $NAME[k]; }
  });
});
".replace("$NAME", binding_ref_name);

          concat_source.add_raw(format!("var {} = require(\"{}\");", binding_ref_name, &importee.stable_id()));

          concat_source.add_raw(import_stmt);

        });
        Some(export_mode)
      } else {
        // There is no need for a non-ESM export kind for determining the export mode.
        None
      }
    } else {
      // The entry module should always be an ECMAScript module, so it is unreachable.
      None
    }
  } else {
    // No need for common chunks to determine the export mode.
    None
  };

  // Runtime module should be placed before the generated `requires` in CJS format.
  // Because, we might need to generate `__toESM(require(...))` that relies on the runtime module.
  let mut module_sources_peekable = module_sources.into_iter().peekable();
  match module_sources_peekable.peek() {
    Some((id, _, _)) if *id == ctx.link_output.runtime.id() => {
      if let (_, _module_id, Some(emitted_sources)) =
        module_sources_peekable.next().expect("Must have module")
      {
        for source in emitted_sources {
          concat_source.add_source(source);
        }
      }
    }
    _ => {}
  }

  let (imports, _) = render_interop_chunk_imports(ctx);

  concat_source.add_raw(imports);

  // chunk content
  module_sources_peekable.for_each(|(_, _, module_render_output)| {
    if let Some(emitted_sources) = module_render_output {
      for source in emitted_sources {
        concat_source.add_source(source);
      }
    }
  });

  if let ChunkKind::EntryPoint { module: entry_id, .. } = ctx.chunk.kind {
    let entry_meta = &ctx.link_output.metas[entry_id];
    match entry_meta.wrap_kind {
      WrapKind::Esm => {
        // init_xxx()
        let wrapper_ref = entry_meta.wrapper_ref.as_ref().unwrap();
        let wrapper_ref_name =
          ctx.link_output.symbols.canonical_name_for(*wrapper_ref, &ctx.chunk.canonical_names);
        concat_source.add_raw(format!("{wrapper_ref_name}();",));
      }
      WrapKind::Cjs => {
        // "export default require_xxx();"
        let wrapper_ref = entry_meta.wrapper_ref.as_ref().unwrap();
        let wrapper_ref_name =
          ctx.link_output.symbols.canonical_name_for(*wrapper_ref, &ctx.chunk.canonical_names);
        concat_source.add_raw(format!("export default {wrapper_ref_name}();\n"));
      }
      WrapKind::None => {}
    }
  }

  let export_mode = export_mode.unwrap_or(OutputExports::Auto);

  let exports = render_chunk_exports(ctx, Some(&export_mode));
  concat_source.add_optional_raw(exports);

  concat_source.add_optional_raw(outro);

  concat_source.add_optional_raw(footer);

  Ok(concat_source)
}
