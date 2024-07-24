use crate::utils::chunk::render_chunk_imports::render_chunk_imports;
use crate::{
  types::generator::GenerateContext,
  utils::chunk::render_chunk_exports::{get_export_items, render_chunk_exports},
};
use rolldown_common::OutputExports;
use rolldown_utils::ecma_script::legitimize_identifier_name;

pub type RenderWrapperResult = (String, String, Vec<(String, bool)>);

pub fn render_wrapper(
  ctx: &mut GenerateContext<'_>,
  export_mode: &OutputExports, // Won't be `OutputExports::Auto`
  use_strict: bool,
  intro: Option<String>,
  outro: Option<String>,
) -> RenderWrapperResult {
  // The `bool` means whether the module is empty
  let mut beginning = String::new();
  let mut ending = String::new();

  // wrapper start
  let export_items = get_export_items(ctx.chunk, ctx.link_output);
  let has_exports = !export_items.is_empty();

  let named_exports = matches!(export_mode, OutputExports::Named);

  let (import_code, externals) = render_chunk_imports(ctx);

  let input_args = render_wrapper_arguments(&externals, has_exports && named_exports);

  beginning.push_str(format!("(function({input_args}) {{\n").as_str());

  if use_strict {
    beginning.push_str("\"use strict\";\n");
  }

  if let Some(intro) = intro {
    if !intro.is_empty() {
      beginning.push_str(format!("\n{intro}\n").as_str());
    }
  }

  beginning.push_str(import_code.as_str());

  // wrapper exports
  if let Some(exports) = render_chunk_exports(ctx, export_mode) {
    ending.push_str(exports.as_str());
  }

  if let Some(outro) = outro {
    if !outro.is_empty() {
      ending.push_str(format!("\n{outro}\n").as_str());
    }
  }

  if named_exports {
    // We need to add `return exports;` here only if using `named`, because the default value is returned when using `default` in `render_chunk_exports`.
    ending.push_str("\nreturn exports;");
  }

  ending.push_str("\n})");

  (beginning, ending, externals)
}

fn render_wrapper_arguments(externals: &[(String, bool)], exports_key: bool) -> String {
  let mut input_args = if exports_key { vec!["exports".to_string()] } else { vec![] };
  externals.iter().for_each(|(external, non_empty)| {
    if *non_empty {
      input_args.push(legitimize_identifier_name(external).to_string());
    }
  });
  input_args.join(", ")
}
