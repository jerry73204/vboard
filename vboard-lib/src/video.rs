use crate::{common::*, config, msg, state::GLOBAL_STATE};

pub(crate) async fn run_video_server(
    config: config::Registration,
    sample_rx: flume::Receiver<msg::Sample>,
) -> Result<()> {
    let video_dir = GLOBAL_STATE.get().await.video_dir(&config.name);

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

    let video_info = to_video_info(&config)?;

    appsrc.set_caps(Some(&video_info.to_caps()?));
    appsrc.set_format(gst::Format::Time);

    // configure sink
    hlssink2.set_property("max-files", 5u32);
    hlssink2.set_property(
        "location",
        &format!("{}/segment%%05d.ts", video_dir.display()),
    );
    hlssink2.set_property(
        "playlist-location",
        &format!("{}/playlist.m3u8", video_dir.display()),
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

fn to_video_info(config: &config::Registration) -> Result<gst_video::VideoInfo> {
    use gst::Fraction;
    use gst_video::{VideoChromaSite, VideoFormat, VideoInfo};

    let config::Registration {
        format,
        width,
        height,
        frame_rate,
        ..
    } = *config;

    let fps = {
        let config::FrameRateFrac(num, deno) = frame_rate;
        Fraction::new(num as i32, deno as i32)
    };

    let video_format = {
        use config::ImageFormat as I;
        use VideoFormat as V;

        match format {
            I::JPEG => V::Encoded,
            I::GRAY8 => V::Gray8,
            I::GRAY16_LE => V::Gray16Le,
            I::GRAY16_BE => V::Gray16Be,
            I::RGB => V::Rgb,
            I::BGR => V::Bgr,
            I::RGBA => V::Rgba,
            I::ARGB => V::Argb,
            I::BGRA => V::Bgra,
            I::ABGR => V::Abgr,
        }
    };

    let chroma_site = {
        use config::ImageFormat as I;
        use VideoChromaSite as S;

        match format {
            I::JPEG => S::JPEG,
            _ => S::NONE,
        }
    };

    let info = VideoInfo::builder(video_format, width as u32, height as u32)
        .fps(fps)
        .chroma_site(chroma_site)
        .build()?;

    Ok(info)
}
