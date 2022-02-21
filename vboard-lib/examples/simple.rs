use futures::future;
use vboard_lib::{FrameRateFrac, ImageFormat, Registration};

#[async_std::main]
async fn main() {
    let _handle = Registration {
        name: "simple".into(),
        format: ImageFormat::RGB,
        frame_rate: FrameRateFrac(10, 1),
        height: 640,
        width: 320,
    }
    .build()
    .await
    .unwrap();
    let () = future::pending().await;
}
