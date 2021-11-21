use crate::{VirtDirEntry, VirtFile, VirtFs};
use async_trait::async_trait;
use std::cell::UnsafeCell;
use std::fs::Metadata;
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use tokio::fs::{DirEntry, File, ReadDir};
use tokio::sync::Notify;
use tokio_stream::Stream;

pub struct RelativeFs {
    root: PathBuf,
    depth: usize,
}

struct ReadDirStream {
    rd: ReadDir,
    w_meta: Option<Pin<Box<dyn Future<Output = (std::io::Result<Metadata>, DirEntry)>>>>,
}

struct VirtEntry {
    entry: DirEntry,
    is_dir: bool,
}

fn escape_join(base: impl Into<PathBuf>, append: &Path, fs_depth: usize) -> PathBuf {
    let mut base = base.into();
    let mut depth = 0usize;

    for component in append.components() {
        match component {
            Component::Prefix(_) => continue, // we can ignore prefixes
            Component::RootDir => {
                for _ in 0..fs_depth {
                    assert!(base.pop());
                }
            }
            Component::CurDir => continue,
            Component::ParentDir => {
                if depth > 0 {
                    base.pop();
                    depth -= 1;
                }
            }
            Component::Normal(segment) => {
                base.push(Path::new(segment));
                depth += 1;
            }
        }
    }

    base
}

impl RelativeFs {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            root: path.into(),
            depth: 0,
        }
    }
}

#[async_trait]
impl VirtFs for RelativeFs {
    async fn read_dir(
        &self,
        path: &Path,
    ) -> std::io::Result<
        Pin<Box<dyn Stream<Item = std::io::Result<Pin<Box<dyn VirtDirEntry>>>> + Unpin>>,
    > {
        tokio::fs::read_dir(escape_join(&self.root, path, self.depth))
            .await
            .map(|stream| {
                Box::pin(ReadDirStream {
                    rd: stream,
                    w_meta: None,
                }) as Pin<Box<dyn Stream<Item = _> + Unpin>>
            })
    }

    async fn open(&self, path: &Path) -> std::io::Result<Pin<Box<dyn VirtFile>>> {
        tokio::fs::File::open(escape_join(&self.root, path, self.depth))
            .await
            .map(|file| Pin::new(Box::new(file) as Box<dyn VirtFile>))
    }

    async fn enter(&self, path: &Path, new_root: bool) -> Self {
        let joined = escape_join(&self.root, path, self.depth);
        if new_root {
            return Self {
                root: joined,
                depth: 0,
            };
        }
        let pd = path
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count()
            - self.depth;
        let npd = joined
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count();
        Self {
            root: joined,
            depth: npd - pd,
        }
    }

    async fn is_absolute(&self, path: &Path) -> PathBuf {
        let mut joined = escape_join(&self.root, path, self.depth);
        let pd = path
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count()
            - self.depth;
        let npd = joined
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count();

        let depth = npd - pd;
        for _ in 0..depth {
            joined.pop();
        }

        joined
    }
}

impl VirtFile for File {}

impl Stream for ReadDirStream {
    type Item = std::io::Result<Pin<Box<dyn VirtDirEntry>>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let (meta, item) = if let Some(meta) = &mut self.w_meta {
                match meta.as_mut().poll(cx) {
                    Poll::Ready(meta) => meta,
                    Poll::Pending => return Poll::Pending,
                }
            } else {
                let item = match self.rd.poll_next_entry(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(err))),
                    Poll::Ready(Ok(None)) => return Poll::Ready(None),
                    Poll::Ready(Ok(Some(item))) => item,
                };
                self.w_meta = Some(Box::pin(async move { (item.metadata().await, item) }));
                continue;
            };
            self.w_meta = None;
            let meta = match meta {
                Ok(meta) => meta,
                Err(err) => return Poll::Ready(Some(Err(err))),
            };
            break match (meta.is_dir(), meta.is_file()) {
                (dir, file) if dir != file => Poll::Ready(Some(Ok(Box::pin(VirtEntry {
                    entry: item,
                    is_dir: dir,
                })))),
                (_, _) => continue,
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[async_trait]
impl VirtDirEntry for VirtEntry {
    async fn is_dir(&self) -> bool {
        self.is_dir
    }

    async fn is_file(&self) -> bool {
        !self.is_dir
    }
}
