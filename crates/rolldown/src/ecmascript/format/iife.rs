use crate::utils::chunk::collect_render_chunk_imports::{
  collect_render_chunk_imports, RenderImportDeclarationSpecifier,
};
use crate::utils::chunk::determine_use_strict::determine_use_strict;
use crate::{
  ecmascript::ecma_generator::RenderedModuleSources,
  types::generator::GenerateContext,
  utils::chunk::render_chunk_exports::{
    determine_export_mode, get_export_items, render_chunk_exports,
  },
};
use rolldown_common::{ChunkKind, OutputExports};
use rolldown_error::DiagnosableResult;
use rolldown_sourcemap::{ConcatSource, RawSource, Source};
use rolldown_utils::ecma_script::legitimize_identifier_name;
use rustc_hash::FxHashMap;

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

  if let Some(banner) = banner {
    concat_source.add_source(Box::new(RawSource::new(banner)));
  }

  // iife wrapper start
  let export_items = get_export_items(ctx.chunk, ctx.link_output);
  let has_exports = !export_items.is_empty();
  // Since before rendering the `determine_export_mode` runs, `unwrap` here won't cause panic.
  // FIXME do not call `determine_export_mode` twice
  let entry_module = match ctx.chunk.kind {
    ChunkKind::EntryPoint { module, .. } => {
      &ctx.link_output.module_table.modules[module].as_ecma().expect("should be ecma module")
    }
    ChunkKind::Common => unreachable!("iife should be entry point chunk"),
  };
  let named_exports = matches!(
    determine_export_mode(&ctx.options.exports, entry_module, &export_items)?,
    OutputExports::Named
  );

  let (import_code, externals) = render_iife_chunk_imports(ctx);

  let (input_args, output_args) =
    render_iife_arguments(&externals, &ctx.options.globals, has_exports && named_exports);

  concat_source.add_source(Box::new(RawSource::new(format!(
    "{}(function({}) {{\n",
    if let Some(name) = &ctx.options.name { format!("var {name} = ") } else { String::new() },
    // TODO handle external imports here.
    input_args
  ))));

  if determine_use_strict(ctx) {
    let mut source= RawSource::new("\"use strict\";".to_string());
    source.indent(2);
    concat_source.add_source(Box::new(source));
  }

  if let Some(intro) = intro {
    let mut intro = RawSource::new(intro);
    intro.indent(2);
    concat_source.add_source(Box::new(intro));
  }

  concat_source.add_source(Box::new({
    let mut source = RawSource::new(import_code);
    source.indent(2);
    source
  }));

  // chunk content
  module_sources.into_iter().for_each(|(_, _, module_render_output)| {
    if let Some(emitted_sources) = module_render_output {
      for source in emitted_sources {
        let mut source = source;
        source.indent(2);
        concat_source.add_source(source);
      }
    }
  });

  // iife exports
  if let Some(exports) = render_chunk_exports(ctx)? {
    let mut exports = RawSource::new(exports);
    exports.indent(2);
    concat_source.add_source(Box::new(exports));
  }

  if let Some(outro) = outro {
    let mut outro = RawSource::new(outro);
    outro.indent(2);
    concat_source.add_source(Box::new(outro));
  }

  if named_exports && has_exports {
    let mut source = RawSource::new("return exports;".to_string());
    source.indent(2);
    // We need to add `return exports;` here only if using `named`, because the default value is returned when using `default` in `render_chunk_exports`.
    concat_source.add_source(Box::new(source));
  }

  // iife wrapper end
  if invoke {
    concat_source.add_source(Box::new(RawSource::new(format!("}})({output_args});"))));
  } else {
    concat_source.add_source(Box::new(RawSource::new("})".to_string())));
  }

  if let Some(footer) = footer {
    concat_source.add_source(Box::new(RawSource::new(footer)));
  }

  Ok(concat_source)
}

// Handling external imports needs to modify the arguments of the wrapper function.
fn render_iife_chunk_imports(ctx: &GenerateContext<'_>) -> (String, Vec<String>) {
  let render_import_stmts =
    collect_render_chunk_imports(ctx.chunk, ctx.link_output, ctx.chunk_graph);

  let mut s = String::new();
  let externals: Vec<String> = render_import_stmts
    .iter()
    .filter_map(|stmt| {
      let require_path_str = &stmt.path;
      match &stmt.specifiers {
        RenderImportDeclarationSpecifier::ImportSpecifier(specifiers) => {
          // Empty specifiers can be ignored in IIFE.
          if specifiers.is_empty() {
            None
          } else {
            let specifiers = specifiers
              .iter()
              .map(|specifier| {
                if let Some(alias) = &specifier.alias {
                  format!("{}: {alias}", specifier.imported)
                } else {
                  specifier.imported.to_string()
                }
              })
              .collect::<Vec<_>>();
            s.push_str(&format!(
              "const {{ {} }} = {};\n",
              specifiers.join(", "),
              legitimize_identifier_name(&stmt.path)
            ));
            Some(require_path_str.to_string())
          }
        }
        RenderImportDeclarationSpecifier::ImportStarSpecifier(alias) => {
          s.push_str(&format!("const {alias} = {};\n", legitimize_identifier_name(&stmt.path)));
          Some(require_path_str.to_string())
        }
      }
    })
    .collect();

  (s, externals)
}

fn render_iife_arguments(
  externals: &[String],
  globals: &FxHashMap<String, String>,
  exports_key: bool,
) -> (String, String) {
  let mut input_args = if exports_key { vec!["exports".to_string()] } else { vec![] };
  let mut output_args = if exports_key { vec!["{}".to_string()] } else { vec![] };
  externals.iter().for_each(|external| {
    input_args.push(legitimize_identifier_name(external).to_string());
    if let Some(global) = globals.get(external) {
      output_args.push(legitimize_identifier_name(global).to_string());
    } else {
      // TODO add warning for missing global
      output_args.push(legitimize_identifier_name(external).to_string());
    }
  });
  (input_args.join(", "), output_args.join(", "))
}
