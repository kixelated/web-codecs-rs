pub struct DecodedFrame {
    inner: web_sys::VideoFrame,
}

impl From<web_sys::VideoFrame> for DecodedFrame {
    fn from(inner: web_sys::VideoFrame) -> Self {
        Self { inner }
    }
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: f64,
    pub keyframe: bool,
}