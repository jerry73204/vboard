use crate::common::*;

pub struct Sample {
    pub bytes: Cow<'static, [u8]>,
    pub dts: Duration,
    pub pts: Duration,
}

pub(crate) struct Registration {
    pub name: Cow<'static, str>,
    pub width: usize,
    pub height: usize,
}
