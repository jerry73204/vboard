use crate::{common::*, routing::SampleHandle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    GRAY8,
    GRAY16_BE,
    GRAY16_LE,
    RGB,
    RGBA,
    ARGB,
    BGR,
    BGRA,
    ABGR,
    JPEG,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Registration {
    pub name: Cow<'static, str>,
    pub format: ImageFormat,
    pub width: usize,
    pub height: usize,
    pub frame_rate: FrameRateFrac,
}

impl Registration {
    pub async fn build(self) -> Result<SampleHandle> {
        crate::launch::register(self).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameRateFrac(pub usize, pub usize);
