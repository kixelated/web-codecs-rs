pub struct ColorSpaceConfig {
    inner: web_sys::VideoColorSpaceInit,
}

impl ColorSpaceConfig {
    pub fn new() -> Self {
        Self {
            inner: web_sys::VideoColorSpaceInit::new(),
        }
    }

    pub fn full_range(mut self, enabled: bool) -> Self {
        self.inner.full_range(enabled);
        self
    }

    pub fn matrix(mut self, matrix: MatrixCoefficients) -> Self {
        self.inner.matrix(matrix.into());
        self
    }

    pub fn primaries(mut self, primaries: ColorPrimaries) -> Self {
        self.inner.primaries(primaries.into());
        self
    }

    pub fn transfer(mut self, transfer: TransferCharacteristics) -> Self {
        self.inner.transfer(transfer.into());
        self
    }
}

impl From<&ColorSpaceConfig> for web_sys::VideoColorSpaceInit {
    fn from(this: &ColorSpaceConfig) -> Self {
        this.inner.clone()
    }
}

pub type MatrixCoefficients = web_sys::VideoMatrixCoefficients;
pub type ColorPrimaries = web_sys::VideoColorPrimaries;
pub type TransferCharacteristics = web_sys::VideoTransferCharacteristics;
