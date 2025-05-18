use napi::{
  bindgen_prelude::{Array, FromNapiValue, Object, Unknown},
  JsObjectValue, JsValue,
};

pub fn downcast_into<T: FromNapiValue + 'static>(o: Unknown) -> napi::Result<T> {
  <T as FromNapiValue>::from_unknown(o)
}

pub fn object_assign(target: &mut Object, source: &Object) -> napi::Result<()> {
  let names = source.get_all_property_names(
    napi::KeyCollectionMode::OwnOnly,
    napi::KeyFilter::AllProperties,
    napi::KeyConversion::KeepNumbers,
  )?;
  let names = Array::from_unknown(names.to_unknown())?;

  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let value = source.get_property::<Unknown, Unknown>(name)?;
      target.set_property::<Unknown, Unknown>(name, value)?;
    }
  }

  Ok(())
}
