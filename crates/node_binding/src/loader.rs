use napi::Result;
use rspack_binding_options::JsLoaderContext;
use rspack_fs_node::ThreadsafeInputNodeFS;

/// Builtin loader runner
#[napi]
pub async fn run_builtin_loader(
  builtin: String,
  options: Option<String>,
  loader_context: JsLoaderContext,
  input_filesystem: ThreadsafeInputNodeFS,
) -> Result<JsLoaderContext> {
  rspack_binding_options::run_builtin_loader(
    builtin,
    options.as_deref(),
    loader_context,
    input_filesystem,
  )
  .await
}
