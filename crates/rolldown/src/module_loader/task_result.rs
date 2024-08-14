use oxc::index::IndexVec;
use rolldown_common::{
  ImportRecordIdx, Module, ModuleIdx, OutputAsset, RawImportRecord, ResolvedId,
};
use rolldown_ecmascript::EcmaAst;
use rolldown_error::BuildDiagnostic;

use crate::types::ast_symbols::AstSymbols;

pub struct NormalModuleTaskResult {
  pub module_idx: ModuleIdx,
  pub resolved_deps: IndexVec<ImportRecordIdx, ResolvedId>,
  pub raw_import_records: IndexVec<ImportRecordIdx, RawImportRecord>,
  pub warnings: Vec<BuildDiagnostic>,
  pub module: Module,
  pub ecma_related: Option<(EcmaAst, AstSymbols)>,
  pub assets: Vec<OutputAsset>,
}
