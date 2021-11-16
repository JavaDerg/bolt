pub mod tkfs;

use async_trait::async_trait;
use std::io::Result;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use tokio_stream::Stream;

#[async_trait]
pub trait VirtFs {
    async fn read_dir(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Stream<Item = Result<Box<dyn VirtDirEntry>>>>>;
    async fn open(&self, path: &Path) -> Result<Box<dyn VirtFile>>;
    async fn enter(&self, path: &Path, new_root: bool) -> Self;
    async fn is_absolute(&self, path: &Path) -> PathBuf;
}

#[async_trait]
pub trait VirtDirEntry {
    async fn is_dir(&self) -> bool;
    async fn is_file(&self) -> bool;
}

#[async_trait]
pub trait VirtFile: AsyncSeek + AsyncRead + AsyncWrite {}
