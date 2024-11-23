pub struct EncodedFrame {
    pub payload: bytes::Bytes,
    pub timestamp: f64,
    pub keyframe: bool,
}
