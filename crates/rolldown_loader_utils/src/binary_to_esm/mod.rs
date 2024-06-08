pub fn binary_to_esm(base64_source: &str) -> anyhow::Result<String> {
  let template = include_str!("to_binary.js").to_string();
  // The `template` provides the transformation tool from base64 to Uint8Array, which is `binary`
  let define_line = format!("const result = __toBinary(\"{base64_source}\");");
  let export_line = "export default result;".to_string();
  Ok([template, define_line, export_line].join("\n"))
}
