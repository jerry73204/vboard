use futures::future;

#[async_std::main]
async fn main() {
    let handle = vboard_lib::register("simple", [640, 480]).await;
    let () = future::pending().await;
}
