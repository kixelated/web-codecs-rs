pub struct VideoColorSpaceConfig {
    inner: web_sys::VideoColorSpaceInit,
}

impl VideoColorSpaceConfig {
    pub fn new() -> Self {
        Self {
            inner: web_sys::VideoColorSpaceInit::new(),
        }
    }

    pub fn full_range(self, enabled: bool) -> Self {
        self.inner.set_full_range(enabled);
        self
    }

    pub fn matrix(self, matrix: VideoMatrixCoefficients) -> Self {
        self.inner.set_matrix(matrix.into());
        self
    }

    pub fn primaries(self, primaries: VideoColorPrimaries) -> Self {
        self.inner.set_primaries(primaries.into());
        self
    }

    pub fn transfer(self, transfer: VideoTransferCharacteristics) -> Self {
        self.inner.set_transfer(transfer.into());
        self
    }
}

impl From<&VideoColorSpaceConfig> for web_sys::VideoColorSpaceInit {
    fn from(this: &VideoColorSpaceConfig) -> Self {
        this.inner.clone()
    }
}

pub type VideoMatrixCoefficients = web_sys::VideoMatrixCoefficients;
pub type VideoColorPrimaries = web_sys::VideoColorPrimaries;
pub type VideoTransferCharacteristics = web_sys::VideoTransferCharacteristics;
