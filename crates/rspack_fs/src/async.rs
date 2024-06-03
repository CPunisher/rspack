use std::path::Path;

use futures::future::BoxFuture;

use crate::Result;

pub trait AsyncWritableFileSystem {
  /// Creates a new, empty directory at the provided path.
  ///
  /// NOTE: If a parent of the given path doesn’t exist, this function is supposed to return an error.
  /// To create a directory and all its missing parents at the same time, use the [`create_dir_all`] function.
  ///
  /// Error:
  /// This function is supposed to return an error in the following situations, but is not limited to just these cases:
  /// - User lacks permissions to create directory at path.
  /// - A parent of the given path doesn’t exist. (To create a directory and all its missing parents at the same time, use the create_dir_all function.)
  /// - Path already exists.
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;

  /// Recursively create a directory and all of its parent components if they are missing.
  fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;

  /// Write a slice as the entire contents of a file.
  /// This function will create a file if it does not exist, and will entirely replace its contents if it does.
  fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> BoxFuture<'_, Result<()>>;

  /// Removes a file from the filesystem.
  fn remove_file<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<()>>;

  /// Removes a directory at this path, after removing all its contents. Use carefully.
  fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>>;
}

pub struct Metadata {
  pub is_dir: bool,
  pub is_file: bool,
  pub is_symlink: bool,
}

pub struct DirEntry {
  pub name: String,
  pub path: String,
  pub metadata: Metadata,
}

pub trait AsyncReadableFileSystem: Send + Sync + std::fmt::Debug {
  /// Read the entire contents of a file into a bytes vector.
  ///
  /// Error: This function will return an error if path does not already exist.
  fn read(&self, file: &Path) -> BoxFuture<'_, Result<Vec<u8>>>;

  fn read_dir(&self, file: &Path) -> BoxFuture<'_, Result<Vec<DirEntry>>>;

  fn metadata(&self, file: &Path) -> BoxFuture<'_, Result<Metadata>>;

  fn symbolic_metadata(&self, file: &Path) -> BoxFuture<'_, Result<Metadata>>;

  fn canonicalize(&self, file: &Path) -> BoxFuture<'_, Result<String>>;

  fn read_to_string(&self, file: &Path) -> BoxFuture<'_, Result<String>> {
    let file = file.to_owned();
    let fut = async move {
      self
        .read(&file)
        .await
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
    };
    Box::pin(fut)
  }
}

/// Async readable and writable file system representation.
pub trait AsyncFileSystem: AsyncReadableFileSystem + AsyncWritableFileSystem {}

// Blanket implementation for all types that implement both [`AsyncReadableFileSystem`] and [`WritableFileSystem`].
impl<T: AsyncReadableFileSystem + AsyncWritableFileSystem> AsyncFileSystem for T {}
