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

    // initialize gstreamer
    static GST_INIT: Once = Once::new();
    GST_INIT.call_once(|| {
        gst::init().expect("unable to initialize gstreamer");
    });

    // build pipeline
    let pipeline = gst::Pipeline::new(None);

    let make_elem = |name: &'static str| {
        gst::ElementFactory::make(name, None)
            .map_err(|_| anyhow!("missing gstreamer element '{}'", name))
    };

    let appsrc = make_elem("appsrc")?;
    let videoconvert = make_elem("videoconvert")?;
    let x264enc = make_elem("x264enc")?;
    let h264parse = make_elem("h264parse")?;
    let hlssink2 = make_elem("hlssink2")?;

    pipeline.add_many(&[&appsrc, &videoconvert, &x264enc, &h264parse, &hlssink2])?;
    gst::Element::link_many(&[&appsrc, &videoconvert, &x264enc, &h264parse, &hlssink2])?;

    // configure appsrc
    let appsrc = appsrc
        .dynamic_cast::<gst_app::AppSrc>()
        .map_err(|_| anyhow!("Cannot cast to AppSrc"))?;

    let video_info =
        gst_video::VideoInfo::builder(gst_video::VideoFormat::Bgrx, width as u32, height as u32)
            .fps(gst::Fraction::new(10, 1))
            .build()?;

    appsrc.set_caps(Some(&video_info.to_caps()?));
    appsrc.set_format(gst::Format::Time);

    // configure sink
    hlssink2.set_property("max-files", 5u32);
    hlssink2.set_property(
        "location",
        &format!("{}/segment%%05d.ts", client_dir.display()),
    );
    hlssink2.set_property(
        "playlist-location",
        &format!("{}/playlist.m3u8", client_dir.display()),
    );

    // forward messags from channel to pipeline
    let forward_fut = spawn_blocking(move || {
        while let Ok(sample) = sample_rx.recv() {
            let msg::Sample { bytes, pts, dts } = sample;
            let mut buffer = gst::Buffer::from_slice(bytes);

            {
                let buffer = buffer.get_mut().unwrap();
                buffer.set_pts(gst::ClockTime::from_nseconds(pts.as_nanos() as u64));
                buffer.set_dts(gst::ClockTime::from_nseconds(dts.as_nanos() as u64));
            }

            appsrc.push_buffer(buffer)?;
        }
        anyhow::Ok(())
    });

    // run gst pipeline
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

    // wait for all tasks to finish
    futures::try_join!(forward_fut, pipeline_fut)?;

    Ok(())
}
