mod common;
mod config;
mod msg;
mod routing;
mod state;
mod utils;
mod video;
mod web;

use crate::{common::*, state::GLOBAL_STATE};
use routing::SampleHandle;

pub async fn register<S>(name: S, hw: [usize; 2]) -> Result<SampleHandle>
where
    S: Into<Cow<'static, str>>,
{
    let name = name.into();
    let [height, width] = hw;

    launch_once();
    let state = GLOBAL_STATE.get().await;

    let sample_handle = state
        .route_handle
        .register(msg::Registration {
            name,
            width,
            height,
        })
        .await?;

    Ok(sample_handle)
}

fn launch_once() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        spawn(async move {
            run_server().await.unwrap();
        });
    });
}

async fn run_server() -> Result<()> {
    let (route_handle, route_future) = routing::run_routing_server();
    let web_future = web::run_web_server();
    state::init(route_handle)?;

    futures::try_join!(spawn(route_future), spawn(web_future))?;

    Ok(())
}
