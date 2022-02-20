use futures::future;

#[async_std::main]
async fn main() {
    let _handle = vboard_lib::register("simple", [480, 640]).await;
    let () = future::pending().await;
}
