use napi::JsString;

pub trait JsStringExt {
  fn into_string(self) -> String;
}

impl<'env> JsStringExt for JsString<'env> {
  fn into_string(self) -> String {
    self
      .into_utf8()
      .expect("Should into utf8")
      .as_str()
      .expect("Should as_str")
      .to_string()
  }
}
