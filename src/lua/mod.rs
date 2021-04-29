use mlua::{FromLuaMulti, Function, Table, ToLuaMulti};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender as MpscSender;
use tokio::sync::oneshot::Sender as OsSender;
use tokio::task::{JoinHandle, LocalSet};

pub struct SyncFunction<A, I, O>
where
    A: 'static + ToLuaMulti + Send,
    I: 'static + FromLuaMulti + Send,
    O: Send,
{
    inner: Arc<InnerSyncFunction<A, I, O>>,
}

struct InnerSyncFunction<A, I, O>
where
    A: 'static + ToLuaMulti + Send,
    I: 'static + FromLuaMulti + Send,
    O: Send,
{
    transformer: Box<dyn Fn(I) -> O + Send + Sync>,
    tx: MpscSender<(A, OsSender<O>)>,
    handle: JoinHandle<()>,
}

impl<A, I, O> SyncFunction<A, I, O>
where
    A: 'static + ToLuaMulti + Send,
    I: 'static + FromLuaMulti + Send,
    O: Send,
{
    pub fn load(src: &str) -> Self {
        let source = String::from(src);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = LocalSet::new().spawn_local(async move {
            let lua = mlua::Lua::new();
            let func: Function = lua.load(source.as_bytes()).eval_async().await.unwrap();
            while let Some((args, ret)) = rx.recv().await {
                let response = func
                    .call_async(args)
                    .await
                    .expect("TODO: This is unhandled atm");
                let _ = ret.send(response);
            }
        });
        Self {
            inner: Arc::new(InnerSyncFunction {
                transformer: Box::new(()),
                tx,
                handle,
            }),
        }
    }

    pub async fn call(&self, args: A) -> O {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.inner.tx.send((args, tx)).unwrap();
        rx.await.expect("TODO")
    }
}

#[deprecated]
pub fn load_fn(src: &'static str) -> Function<'static> {
    // SAFETY: There is none; TODO: Not leak memory?!
    let lua = Box::leak(Box::new(mlua::Lua::new()));
    lua.load(src.as_bytes()).eval().unwrap()
}
