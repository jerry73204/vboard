use crate::{common::*, msg, msg::Registration, state::GLOBAL_STATE};

pub(crate) async fn run_video_server(
    config: Registration,
    sample_rx: flume::Receiver<msg::Sample>,
) -> Result<()> {
    let Registration {
        name,
        width,
        height,
    } = config;
    let client_dir = GLOBAL_STATE.get().await.client_dir(&name);

    // generate launch string
    let launch = format!(
        "appsrc name=appsrc ! \
            video/x-raw, format=RGB, width={}, height={}, framerate=10/1 ! \
            videoconvert ! \
            x264enc ! \
            h264parse ! \
            hlssink2 max-files=5 location={}/segment%%05d.ts playlist-location={}/playlist.m3u8",
        width,
        height,
        client_dir.display(),
        client_dir.display(),
    );

    // initialize gstreamer
    static GST_INIT: Once = Once::new();
    GST_INIT.call_once(|| {
        gst::init().expect("unable to initialize gstreamer");
    });

    // initialize pipeline
    let pipeline = gst::parse_launch(&launch)?
        .dynamic_cast::<gst::Pipeline>()
        .map_err(|_| anyhow!("Cannot cast launch string to Pipeline type"))?;

    let appsrc = pipeline
        .by_name("appsrc")
        .ok_or_else(|| anyhow!(r#"Cannot find element named "appsrc""#))?
        .dynamic_cast::<gst_app::AppSrc>()
        .map_err(|_| anyhow!("Cannot cast to AppSrc"))?;

    let forward_fut = spawn_blocking(move || {
        let since = Instant::now();

        while let Ok(sample) = sample_rx.recv() {
            let msg::Sample { bytes } = sample;

            let mut buffer = gst::Buffer::from_slice(bytes);

            {
                let buffer = buffer.get_mut().unwrap();
                let nanos = since.elapsed().as_nanos() as u64;

                buffer.set_pts(gst::ClockTime::from_nseconds(nanos));
                buffer.set_dts(gst::ClockTime::from_nseconds(nanos));
            }

            appsrc.push_buffer(buffer)?;
        }
        anyhow::Ok(())
    });

    let pipeline_fut = spawn_blocking(move || {
        pipeline.set_state(gst::State::Playing)?;

        let bus = pipeline
            .bus()
            .ok_or_else(|| anyhow!("Pipeline without bus. Shouldn't happen!"))?;

        for msg in bus.iter_timed(None) {
            use gst::MessageView as M;

            match msg.view() {
                M::Eos(..) => break,
                M::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("gst error: {:?}", err));
                }
                _ => {}
            }
        }

        pipeline.set_state(gst::State::Null)?;
        Ok(())
    });

    futures::try_join!(forward_fut, pipeline_fut)?;

    Ok(())
}
