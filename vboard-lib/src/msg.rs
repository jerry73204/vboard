use crate::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sample {
    pub bytes: Cow<'static, [u8]>,
    pub dts: Duration,
    pub pts: Duration,
}
