pub struct ColorSpaceConfig {
    inner: web_sys::VideoColorSpaceInit,
}

impl Default for ColorSpaceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpaceConfig {
    pub fn new() -> Self {
        Self {
            inner: web_sys::VideoColorSpaceInit::new(),
        }
    }

    pub fn full_range(self, enabled: bool) -> Self {
        self.inner.set_full_range(enabled);
        self
    }

    pub fn matrix(self, matrix: MatrixCoefficients) -> Self {
        self.inner.set_matrix(matrix);
        self
    }

    pub fn primaries(self, primaries: ColorPrimaries) -> Self {
        self.inner.set_primaries(primaries);
        self
    }

    pub fn transfer(self, transfer: TransferCharacteristics) -> Self {
        self.inner.set_transfer(transfer);
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
