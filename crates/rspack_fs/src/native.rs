use std::{fs, path::Path};

use super::{
  cfg_async,
  sync::{ReadableFileSystem, WritableFileSystem},
  Error, Result,
};
use crate::r#async::{DirEntry, Metadata};

pub struct NativeFileSystem;

impl WritableFileSystem for NativeFileSystem {
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir(dir.as_ref()).map_err(Error::from)
  }

  fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir_all(dir.as_ref()).map_err(Error::from)
  }

  fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()> {
    fs::write(file.as_ref(), data.as_ref()).map_err(Error::from)
  }
}

impl ReadableFileSystem for NativeFileSystem {
  fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>> {
    fs::read(file.as_ref()).map_err(Error::from)
  }
}

cfg_async! {
  use futures::future::BoxFuture;

  use crate::{AsyncReadableFileSystem, AsyncWritableFileSystem};
  #[derive(Debug)]
  pub struct AsyncNativeFileSystem;

  impl AsyncWritableFileSystem for AsyncNativeFileSystem {
    fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::create_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(
      &self,
      file: P,
      data: D,
    ) -> BoxFuture<'_, Result<()>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let data = data.as_ref().to_vec();
      let fut = async move { tokio::fs::write(file, data).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_file<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<()>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_file(file).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
      let dir = dir.as_ref().to_string_lossy().to_string();
      let fut = async move { tokio::fs::remove_dir_all(dir).await.map_err(Error::from) };
      Box::pin(fut)
    }
  }

  impl AsyncReadableFileSystem for AsyncNativeFileSystem {
    fn read(&self, file: &Path) -> BoxFuture<'_, Result<Vec<u8>>> {
      let file = file.to_string_lossy().to_string();
      let fut = async move { tokio::fs::read(file).await.map_err(Error::from) };
      Box::pin(fut)
    }

    fn read_dir(&self, file: &Path) -> BoxFuture<'_, Result<Vec<DirEntry>>> {
      let file = file.to_string_lossy().to_string();
      let fut = async move {
        let mut dir = tokio::fs::read_dir(file).await?;
        let mut dir_entries = Vec::new();
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path().to_string_lossy().to_string();
            let metadata = entry.metadata().await?;
            dir_entries.push(DirEntry {
                path,
                metadata: Metadata {
                    is_dir: metadata.is_dir(),
                    is_file: metadata.is_file(),
                },
            });
        }
        Ok(dir_entries)
      };
      Box::pin(fut)
    }

    fn metadata(&self, file: &Path) -> BoxFuture<'_, Result<Metadata>> {
      let file = file.to_string_lossy().to_string();
      let fut = async move {
        tokio::fs::metadata(file)
          .await
          .map_err(Error::from)
          .map(|metadata| Metadata {
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
          })
      };
      Box::pin(fut)
    }
  }
}
