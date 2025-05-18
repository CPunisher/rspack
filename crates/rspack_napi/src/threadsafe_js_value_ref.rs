use std::{
  marker::PhantomData,
  sync::{Arc, Mutex},
};

use napi::{bindgen_prelude::*, Ref};

use crate::JsCallback;

struct ThreadsafeJsValueRefHandle<'env, T: JsValue<'env>> {
  value_ref: Arc<Mutex<Ref<T>>>,
  drop_handle: JsCallback<Box<dyn FnOnce(Env)>>,
  _marker: PhantomData<&'env ()>,
}

impl<'env, T: JsValue<'env>> ThreadsafeJsValueRefHandle<'env, T> {
  fn new(env: Env, js_ref: Ref<T>) -> Result<Self> {
    Ok(Self {
      value_ref: Arc::new(Mutex::new(js_ref)),
      drop_handle: unsafe { JsCallback::new(env.raw()) }?,
      _marker: PhantomData,
    })
  }
}

impl<'env, T: JsValue<'env>> Drop for ThreadsafeJsValueRefHandle<'env, T> {
  fn drop(&mut self) {
    let value_ref = self.value_ref.clone();
    self.drop_handle.call(Box::new(move |env| {
      let _ = value_ref
        .lock()
        .expect("should lock `value_ref`")
        .unref(&env);
    }))
  }
}

pub struct ThreadsafeJsValueRef<'env, T: JsValue<'env>> {
  inner: Arc<ThreadsafeJsValueRefHandle<'env, T>>,
}

unsafe impl<'env, T: JsValue<'env>> Send for ThreadsafeJsValueRef<'env, T> {}
unsafe impl<'env, T: JsValue<'env>> Sync for ThreadsafeJsValueRef<'env, T> {}

impl<'env, T: JsValue<'env>> Clone for ThreadsafeJsValueRef<'env, T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<'env, T: FromNapiValue + JsValue<'env>> FromNapiValue for ThreadsafeJsValueRef<'env, T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Self::new(Env::from(env), unsafe {
      T::from_napi_value(env, napi_val)
    }?)
  }
}

impl<'env, T: FromNapiValue + JsValue<'env>> ToNapiValue for ThreadsafeJsValueRef<'env, T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    val
      .get(Env::from(env))
      .and_then(|v| unsafe { T::to_napi_value(env, v) })
  }
}

impl<'env, T: FromNapiValue + JsValue<'env>> ThreadsafeJsValueRef<'env, T> {
  pub fn new(env: Env, value: T) -> Result<Self> {
    let js_ref = Ref::new(&env, &value)?;

    Ok(Self {
      inner: Arc::new(ThreadsafeJsValueRefHandle::new(env, js_ref)?),
    })
  }

  pub fn get(&self, env: Env) -> Result<T> {
    self
      .inner
      .value_ref
      .lock()
      .expect("should lock `value_ref`")
      .get_value(&env)
  }
}
