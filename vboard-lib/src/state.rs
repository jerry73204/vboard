use crate::{common::*, routing::RouterHandle, utils};
use anyhow::Context;
use async_once_watch::OnceWatch;
use dashmap::DashMap;

pub struct GlobalState {
    pub instance_id: String,
    pub working_dir: PathBuf,
    pub videos: DashMap<Cow<'static, str>, VideoContext>,
    pub route_handle: RouterHandle,
}

impl GlobalState {
    pub fn client_dir(&self, client_name: &str) -> PathBuf {
        let dir_name = utils::percent_encode(client_name);
        self.working_dir.join(dir_name)
    }
}

pub struct VideoContext {
    pub name: Cow<'static, str>,
    pub height: usize,
    pub width: usize,
}

pub static GLOBAL_STATE: Lazy<OnceWatch<GlobalState>> = Lazy::new(OnceWatch::new);

pub fn init(route_handle: RouterHandle) -> Result<()> {
    let instance_id = format!(
        "{}-{:08}",
        Local::now().format("%Y-%b-%d-%H-%M-%S%.3f%z"),
        process::id()
    );

    let working_dir = dirs::cache_dir()
        .expect("unable to locale cache dir")
        .join(env!("CARGO_PKG_NAME"))
        .join(&instance_id);

    fs::create_dir_all(&working_dir).with_context(|| "unable to create working directory")?;

    let state = GlobalState {
        instance_id,
        working_dir,
        videos: DashMap::new(),
        route_handle,
    };

    GLOBAL_STATE
        .set(state)
        .map_err(|_| anyhow!("internal error: the global state ins set more than once"))?;

    Ok(())
}
