use oxc::allocator::Allocator;
use oxc::ast::ast::{ImportDeclaration, Statement};
use oxc::ast::VisitMut;
use oxc::codegen::CodeGenerator;
use oxc::parser::Parser;
use oxc::span::{Atom, SourceType};
use std::path::Path;
use url::Url;

struct HttpResolveTransformer<'a> {
  pub base_url: Url,
  pub url_path: String,
  pub allocator: &'a Allocator,
}

impl<'a> HttpResolveTransformer<'a> {
  pub fn new(allocator: &'a Allocator, base_url: &str) -> Self {
    let base_url = Url::parse(base_url).unwrap();
    Self { allocator, base_url: base_url.clone(), url_path: base_url.path().to_string() }
  }
}

impl<'a> VisitMut<'a> for HttpResolveTransformer<'a> {
  fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
    let source = decl.source.value.as_str();
    let path = Path::new(self.url_path.as_str());
    let url = path.join("..").join(source);
    let url = url.to_str().unwrap();
    let resolved_url = self.base_url.join(&url).unwrap();
    println!("Resolved URL: {}", resolved_url);
    let resolved_url: &'a str = self.allocator.alloc_str(resolved_url.as_str());
    decl.source.value = Atom::from(resolved_url);
  }
}

pub fn transform_source(source: &str, base_url: &str) -> anyhow::Result<String> {
  let allocator = Allocator::default();
  let url = Url::parse(base_url)?;
  let path = url.path().to_string();
  let path = Path::new(&path);
  let module_type = if let Ok(result) = SourceType::from_path(path) {
    result
  } else {
    anyhow::bail!("Unknown module type")
  };
  let mut program = Parser::new(&allocator, source, module_type).parse().program;
  let mut transformer = HttpResolveTransformer::new(&allocator, base_url);
  program.body.iter_mut().for_each(|stmt| {
    match stmt {
        Statement::ImportDeclaration(decl) => {
          transformer.visit_import_declaration(decl);
        }
        _ => {}
    }
  });
  let codegen = CodeGenerator::new();
  let result = codegen.build(&program).source_text;
  Ok(result)
}
