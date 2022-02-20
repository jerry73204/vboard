use crate::{
    common::*,
    msg::{self, Registration},
    state::GLOBAL_STATE,
};

pub(crate) fn run_routing_server() -> (RouterHandle, BoxFuture<'static, Result<()>>) {
    let (tx, rx) = flume::unbounded();
    let future = async move {
        while let Ok(future) = rx.recv_async().await {
            future.await?;
        }

        anyhow::Ok(())
    }
    .boxed();
    let handle = RouterHandle { sender: tx };

    (handle, future)
}

pub struct RouterHandle {
    sender: flume::Sender<BoxFuture<'static, Result<()>>>,
}

impl RouterHandle {
    pub(crate) async fn register(&self, config: Registration) -> Result<SampleHandle> {
        let name = config.name.clone();
        let ok = GLOBAL_STATE.get().await.clients.insert(name.clone());
        ensure!(ok, "the client '{}' is already registered", name);

        let (sample_tx, sample_rx) = flume::unbounded();
        let future = crate::video::run_video_server(config, sample_rx).boxed();

        let ok = self.sender.try_send(future).is_ok();
        ensure!(ok, "unable to start video server for client '{}'", name);

        let handle = SampleHandle { sender: sample_tx };

        Ok(handle)
    }
}

pub struct SampleHandle {
    sender: flume::Sender<msg::Sample>,
}

impl SampleHandle {
    pub fn send(&self, sample: msg::Sample) -> Result<(), msg::Sample> {
        self.sender.try_send(sample).map_err(|err| err.into_inner())
    }
}
