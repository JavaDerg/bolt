pub mod tkfs;

use async_trait::async_trait;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use tokio_stream::Stream;

#[async_trait]
pub trait VirtFs: Unpin {
    async fn read_dir(
        &self,
        path: &Path,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Pin<Box<dyn VirtDirEntry>>>> + Unpin>>>;
    async fn open(&self, path: &Path) -> Result<Pin<Box<dyn VirtFile>>>;
    async fn enter(&self, path: &Path, new_root: bool) -> Self;
    async fn is_absolute(&self, path: &Path) -> PathBuf;
}

#[async_trait]
pub trait VirtDirEntry: Unpin {
    async fn is_dir(&self) -> bool;
    async fn is_file(&self) -> bool;
}

#[async_trait]
pub trait VirtFile: AsyncSeek + AsyncRead + AsyncWrite + Unpin {}
