pub fn image_mime(extension: &str) -> Option<&'static str> {
  match extension {
    // Only support common image formats for now
    "bmp" => Some("image/bmp"),
    "gif" => Some("image/gif"),
    "jpg" | "jpeg" => Some("image/jpeg"),
    "png" => Some("image/png"),
    "svg" => Some("image/svg+xml"),
    "webp" => Some("image/webp"),
    _ => None,
  }
}
