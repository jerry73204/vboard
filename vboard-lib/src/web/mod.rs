mod template;

use crate::{common::*, state::GLOBAL_STATE};
use tide::{Body, Error, Request, Response, StatusCode};

pub async fn run_web_server() -> Result<()> {
    tide::log::start();

    let mut app = tide::new();
    app.at("/").get(index_page);
    app.at("/video/:name").get(video_page);
    app.at("/file/:name/:file").get(file_page);
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}

async fn index_page(_: Request<()>) -> tide::Result {
    let state = GLOBAL_STATE.get().await;
    let names: Vec<_> = state
        .videos
        .iter()
        .map(|pair| pair.value().name.clone())
        .collect();

    let template = template::Index {
        title: "test".into(),
        names,
    };

    Ok(template.into())
}

async fn video_page(req: Request<()>) -> tide::Result {
    let name = req.param("name")?;

    let state = GLOBAL_STATE.get().await;
    let context = state.videos.get(name).ok_or_else(|| {
        Error::new(
            StatusCode::NotFound,
            anyhow!("the video '{}' does not exist", name),
        )
    })?;

    let template = template::Video {
        name: context.name.clone(),
        height: context.height,
        width: context.width,
    };

    Ok(template.into())
}

async fn file_page(req: Request<()>) -> tide::Result {
    let name = req.param("name")?;
    let file_name = req.param("file")?;

    if file_name == "playlist.m3u8" {
        let client_dir = GLOBAL_STATE.get().await.client_dir(name);
        let path = client_dir.join(file_name);

        let mut res = Response::new(StatusCode::Ok);
        let body = Body::from_file(&path).await?;
        res.set_body(body);

        Ok(res)
    } else if file_name.ends_with(".ts") {
        todo!();
    } else {
        Err(Error::new(
            StatusCode::NotFound,
            anyhow!("invalid file name"),
        ))
    }
}
