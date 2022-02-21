use crate::{common::*, config::Registration, routing::SampleHandle, state::GLOBAL_STATE};

pub(crate) async fn register(config: Registration) -> Result<SampleHandle> {
    launch_once();
    let state = GLOBAL_STATE.get().await;
    let sample_handle = state.route_handle.register(config).await?;
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
    let (route_handle, route_future) = crate::routing::run_routing_server();
    let web_future = crate::web::run_web_server();
    crate::state::init(route_handle)?;

    futures::try_join!(spawn(route_future), spawn(web_future))?;

    Ok(())
}
