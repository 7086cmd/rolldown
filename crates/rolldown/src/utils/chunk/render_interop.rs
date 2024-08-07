use crate::types::generator::GenerateContext;
use crate::utils::chunk::collect_render_chunk_imports::{
  collect_render_chunk_imports, RenderImportDeclarationSpecifier, RenderImportStmt,
};
use rolldown_common::OutputFormat;
use rolldown_utils::ecma_script::legitimize_identifier_name;

/// Handling external imports needs to modify the arguments of the wrapper function.
/// Here, we combine the CJS, and IIFE cases. When we are supporting UMD, and AMD, we can just use the IIFE case.
/// This is also helpful to support `output.interop` option.
pub fn render_interop_chunk_imports(ctx: &GenerateContext<'_>) -> (String, Vec<String>) {
  let is_cjs = matches!(ctx.options.format, OutputFormat::Cjs);

  let render_import_stmts =
    collect_render_chunk_imports(ctx.chunk, ctx.link_output, ctx.chunk_graph);

  let mut s = String::new();

  // Here we should handle the imports from other chunks. Only CJS needs this.
  if is_cjs {
    // render imports from other chunks
    ctx.chunk.imports_from_other_chunks.iter().for_each(|(exporter_id, _items)| {
      let importee_chunk = &ctx.chunk_graph.chunks[*exporter_id];
      s.push_str(&format!(
        "const {} = require('{}');\n",
        ctx.chunk.require_binding_names_for_other_chunks[exporter_id],
        ctx.chunk.import_path_for(importee_chunk)
      ));
    });
  }

  // In pure CJS, we need to use `require` within the chunk.
  // In other cases, the inputted value is the argument of the wrapper function.
  // Here, these formats (including IIFE, CJS, AMD, and UMD) shares the same logic.
  let externals: Vec<String> = render_import_stmts
    .iter()
    .filter_map(|stmt| {
      if !stmt.is_external {
        return None;
      }
      let require_path_str =
        if is_cjs { format!("require(\"{}\")", &stmt.path) } else { stmt.path.to_string() };
      match &stmt.specifiers {
        RenderImportDeclarationSpecifier::ImportSpecifier(specifiers) => {
          // Empty specifiers can be ignored in wrapper function, but can't be ignored in CJS.
          // TODO amd needs the export key still.
          if specifiers.is_empty() {
            if is_cjs {
              s.push_str(&format!("{require_path_str};\n"));
            }
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
              format_require_method(ctx, stmt, require_path_str.as_str())
            ));
            Some(require_path_str.to_string())
          }
        }
        RenderImportDeclarationSpecifier::ImportStarSpecifier(alias) => {
          s.push_str(&format!(
            "const {alias} = {};\n",
            format_require_method(ctx, stmt, require_path_str.as_str())
          ));
          Some(require_path_str.to_string())
        }
      }
    })
    .collect();

  (s, externals)
}

fn format_require_method(
  ctx: &GenerateContext<'_>,
  stmt: &RenderImportStmt,
  require_path_str: &str,
) -> String {
  if matches!(ctx.options.format, OutputFormat::Cjs) {
    if stmt.is_external {
      let to_esm_fn_name = &ctx.chunk.canonical_names[&ctx
        .link_output
        .symbols
        .par_canonical_ref_for(ctx.link_output.runtime.resolve_symbol("__toESM"))];

      format!("{to_esm_fn_name}({require_path_str})")
    } else {
      require_path_str.to_string()
    }
  } else {
    legitimize_identifier_name(&stmt.path).to_string()
  }
}
