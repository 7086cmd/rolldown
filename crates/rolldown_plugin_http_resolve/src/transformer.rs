use oxc::allocator::Allocator;
use oxc::ast::ast::ImportDeclaration;
use oxc::ast::Visit;
use oxc::parser::Parser;
use oxc::span::SourceType;
use url::Url;

struct HttpResolveTransformer {
    base_url: Url,
    url_path: String,
}

impl Visit for HttpResolveTransformer {
    fn visit_import_declaration<'ast>(&mut self, decl: &ImportDeclaration<'ast>) {
        let source = decl.source.value.as_str();
        // let path = self.url_path;
    }
}
