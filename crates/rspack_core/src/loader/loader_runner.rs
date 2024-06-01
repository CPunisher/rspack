use std::sync::Arc;

use rspack_fs::AsyncReadableFileSystem;
pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext};
use rspack_util::source_map::SourceMapKind;

use crate::{CompilerOptions, Context, ModuleIdentifier, ResolverFactory, SharedPluginDriver};

#[derive(Debug, Clone)]
pub struct CompilerContext {
  pub fs: Arc<dyn AsyncReadableFileSystem + Send + Sync>,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: ModuleIdentifier,             // current module
  pub module_context: Option<Box<Context>>, // current module context
  pub module_source_map_kind: SourceMapKind,
  pub plugin_driver: SharedPluginDriver,
}

pub type LoaderRunnerContext = CompilerContext;

pub type BoxLoader = Arc<dyn Loader<LoaderRunnerContext>>;
