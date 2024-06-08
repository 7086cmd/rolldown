pub fn base64_to_esm(source: &str) ->  anyhow::Result<String> {
  Ok(["export default '", source, "';"].concat())
}
