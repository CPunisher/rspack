#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;

use std::sync::Arc;

use compiler::{Compiler, CompilerState, CompilerStateGuard};
use napi::{bindgen_prelude::*, tokio::sync::Mutex};
use plugins::{JsHooksAdapterPlugin, RegisterJsTaps};
use resolver_factory::JsResolverFactory;
use rspack_core::{Compilation, PluginExt};

mod compiler;
mod diagnostic;
mod panic;
mod plugins;
mod resolver_factory;

pub mod trace;

pub use diagnostic::*;
use rspack_binding_values::*;
use rspack_error::Diagnostic;
use rspack_fs::IntermediateFileSystem;
use rspack_fs_node::{NodeFileSystem, ThreadsafeNodeFS};

#[napi]
pub struct Rspack {
  js_plugin: JsHooksAdapterPlugin,
  compiler: Mutex<Compiler>,
  state: CompilerState,
}

#[napi]
impl Rspack {
  #[allow(clippy::too_many_arguments)]
  #[napi(constructor)]
  pub fn new(
    env: Env,
    compiler_path: String,
    options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    intermediate_filesystem: Option<ThreadsafeNodeFS>,
    mut resolver_factory_reference: Reference<JsResolverFactory>,
  ) -> Result<Self> {
    #[cfg(target_family = "wasm")]
    {
      use std::io::Write;
      tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_target(false)
        .init();
      std::panic::set_hook(Box::new(|info: &std::panic::PanicHookInfo| {
        let _ = writeln!(std::io::stderr(), "{}", info);
      }));
      if let Ok(_) = std::fs::metadata(std::env::current_dir().unwrap()) {};
    };

    tracing::info!("raw_options: {:#?}", &options);

    let mut plugins = Vec::new();
    let js_plugin = JsHooksAdapterPlugin::from_js_hooks(env, register_js_taps)?;
    plugins.push(js_plugin.clone().boxed());
    for bp in builtin_plugins {
      bp.append_to(env, &mut plugins)
        .map_err(|e| Error::from_reason(format!("{e}")))?;
    }

    let compiler_options: rspack_core::CompilerOptions = options
      .try_into()
      .map_err(|e| Error::from_reason(format!("{e}")))?;

    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let resolver_factory =
      (*resolver_factory_reference).get_resolver_factory(compiler_options.resolve.clone());
    let loader_resolver_factory = (*resolver_factory_reference)
      .get_loader_resolver_factory(compiler_options.resolve_loader.clone());

    let intermediate_filesystem: Option<Arc<dyn IntermediateFileSystem>> =
      if let Some(fs) = intermediate_filesystem {
        Some(Arc::new(NodeFileSystem::new(fs).map_err(|e| {
          Error::from_reason(format!("Failed to create intermediate filesystem: {e}",))
        })?))
      } else {
        None
      };

    let rspack = rspack_core::Compiler::new(
      compiler_path,
      compiler_options,
      plugins,
      rspack_binding_values::buildtime_plugins::buildtime_plugins(),
      Some(Arc::new(NodeFileSystem::new(output_filesystem).map_err(
        |e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)),
      )?)),
      intermediate_filesystem,
      None,
      Some(resolver_factory),
      Some(loader_resolver_factory),
    );

    Ok(Self {
      compiler: Mutex::new(Compiler::from(rspack)),
      state: CompilerState::init(),
      js_plugin,
    })
  }

  // #[napi]
  // pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
  //   self.js_plugin.set_non_skippable_registers(kinds)
  // }

  #[napi]
  pub async fn build_async(&self) -> napi::Result<()> {
    tracing::info!("start building");
    let mut compiler = self.compiler.try_lock().unwrap();
    compiler.build().await.map_err(|e| {
      Error::new(
        napi::Status::GenericFailure,
        print_error_diagnostic(e, compiler.options.stats.colors),
      )
    })?;
    tracing::info!("build ok");
    Ok(())
  }

  // /// Build with the given option passed to the constructor
  // #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  // pub fn build(&mut self, env: Env, reference: Reference<Rspack>, f: Function) -> Result<()> {
  //   unsafe {
  //     self.run(env, reference, |compiler, _guard| {
  //       callbackify(env, f, async move {
  //         compiler.build().await.map_err(|e| {
  //           Error::new(
  //             napi::Status::GenericFailure,
  //             print_error_diagnostic(e, compiler.options.stats.colors),
  //           )
  //         })?;
  //         tracing::info!("build ok");
  //         drop(_guard);
  //         Ok(())
  //       })
  //     })
  //   }
  // }

  // /// Rebuild with the given option passed to the constructor
  // #[napi(
  //   ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  // )]
  // pub fn rebuild(
  //   &mut self,
  //   env: Env,
  //   reference: Reference<Rspack>,
  //   changed_files: Vec<String>,
  //   removed_files: Vec<String>,
  //   f: Function,
  // ) -> Result<()> {
  //   use std::collections::HashSet;

  //   unsafe {
  //     self.run(env, reference, |compiler, _guard| {
  //       callbackify(env, f, async move {
  //         compiler
  //           .rebuild(
  //             HashSet::from_iter(changed_files.into_iter()),
  //             HashSet::from_iter(removed_files.into_iter()),
  //           )
  //           .await
  //           .map_err(|e| {
  //             Error::new(
  //               napi::Status::GenericFailure,
  //               print_error_diagnostic(e, compiler.options.stats.colors),
  //             )
  //           })?;
  //         tracing::info!("rebuild ok");
  //         drop(_guard);
  //         Ok(())
  //       })
  //     })
  //   }
  // }
}

// impl Rspack {
//   /// Run the given function with the compiler.
//   ///
//   /// ## Safety
//   /// 1. The caller must ensure that the `Compiler` is not moved or dropped during the lifetime of the callback.
//   /// 2. `CompilerStateGuard` should and only be dropped so soon as each `Compiler` is free of use.
//   ///    Accessing `Compiler` beyond the lifetime of `CompilerStateGuard` would lead to potential race condition.
//   unsafe fn run<R>(
//     &mut self,
//     env: Env,
//     reference: Reference<Rspack>,
//     f: impl FnOnce(&'static mut Compiler, CompilerStateGuard) -> Result<R>,
//   ) -> Result<R> {
//     if self.state.running() {
//       return Err(concurrent_compiler_error());
//     }
//     let _guard = self.state.enter();
//     let mut compiler = reference.share_with(env, |s| {
//       // SAFETY: The mutable reference to `Compiler` is exclusive. It's guaranteed by the running state guard.
//       Ok(unsafe { s.compiler.as_mut().get_unchecked_mut() })
//     })?;

//     self.cleanup_last_compilation(&compiler.compilation);

//     // SAFETY:
//     // 1. `Compiler` is pinned and stored on the heap.
//     // 2. `JsReference` (NAPI internal mechanism) keeps `Compiler` alive until its instance getting garbage collected.
//     f(
//       unsafe { std::mem::transmute::<&mut Compiler, &'static mut Compiler>(*compiler) },
//       _guard,
//     )
//   }

//   fn cleanup_last_compilation(&self, compilation: &Compilation) {
//     let compilation_id = compilation.id();

//     JsCompilationWrapper::cleanup_last_compilation(compilation_id);
//     JsModuleWrapper::cleanup_last_compilation(compilation_id);
//     JsChunkWrapper::cleanup_last_compilation(compilation_id);
//     JsChunkGroupWrapper::cleanup_last_compilation(compilation_id);
//     JsDependencyWrapper::cleanup_last_compilation(compilation_id);
//     JsDependenciesBlockWrapper::cleanup_last_compilation(compilation_id);
//   }
// }

fn print_error_diagnostic(e: rspack_error::Error, colored: bool) -> String {
  Diagnostic::from(e)
    .render_report(colored)
    .expect("should print diagnostics")
}
