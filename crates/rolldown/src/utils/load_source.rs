use rolldown_common::{side_effects::HookSideEffects, ModuleType, ResolvedPath};
use rolldown_plugin::{HookLoadArgs, PluginDriver};
use rolldown_sourcemap::SourceMap;
use sugar_path::SugarPath;

pub async fn load_source(
  plugin_driver: &PluginDriver,
  resolved_path: &ResolvedPath,
  module_type: ModuleType,
  fs: &dyn rolldown_fs::FileSystem,
  sourcemap_chain: &mut Vec<SourceMap>,
  side_effects: &mut Option<HookSideEffects>,
) -> anyhow::Result<String> {
  let source =
    if let Some(r) = plugin_driver.load(&HookLoadArgs { id: &resolved_path.path }).await? {
      if let Some(map) = r.map {
        sourcemap_chain.push(map);
      }
      if let Some(v) = r.side_effects {
        *side_effects = Some(v);
      }
      r.code
    } else if resolved_path.ignored {
      String::new()
    } else {
      match module_type {
        ModuleType::Base64 | ModuleType::Binary => {
          rolldown_utils::base64::to_standard_base64(fs.read(resolved_path.path.as_path())?)
        }
        ModuleType::DataUrl => {
          // let extension: &str = resolved_path.path.extension().unwrap().to_str().unwrap(); DO NOT USE `extension` method
          let extension: &str = resolved_path.path.split('.').last().unwrap();
          if !["png", "jpg", "jpeg", "gif", "webp", "ico", "svg"].contains(&extension) {
            return Err(anyhow::anyhow!("Unsupported image extension: {}", resolved_path.path));
          }
          let mime = rolldown_utils::mime::image_mime(extension).unwrap();
          let content: String = if extension == "svg" {
            fs.read_to_string(resolved_path.path.as_path())?
          } else {
            rolldown_utils::base64::to_url_safe_base64(fs.read(resolved_path.path.as_path())?)
          };
          format!("data:{mime};base64,{content}")
        }
        _ => fs.read_to_string(resolved_path.path.as_path())?,
      }
    };
  Ok(source)
}
