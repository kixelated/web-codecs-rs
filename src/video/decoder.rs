use tokio::sync::{mpsc, watch};
use wasm_bindgen::prelude::*;

use super::{ColorSpaceConfig, DecodedFrame, EncodedFrame};
use crate::Error;

pub struct Decoder {
    inner: web_sys::VideoDecoder,

    closed: watch::Receiver<Result<(), Error>>,
    frames: mpsc::UnboundedReceiver<DecodedFrame>,

    // These are held to avoid dropping them.
    #[allow(dead_code)]
    on_error: Closure<dyn FnMut(JsValue)>,
    #[allow(dead_code)]
    on_frame: Closure<dyn FnMut(JsValue)>,
}

impl Decoder {
    pub fn new() -> Result<Self, Error> {
        let (frames_tx, frames_rx) = mpsc::unbounded_channel();
        let (closed_tx, closed_rx) = watch::channel(Ok(()));

        let on_error = Closure::wrap(Box::new(move |e: JsValue| {
            closed_tx.send_modify(|closed| {
                *closed = Err(Error::from(e));
            });
        }) as Box<dyn FnMut(_)>);

        let on_frame = Closure::wrap(Box::new(move |e: JsValue| {
            let frame: web_sys::VideoFrame = e.unchecked_into();
            let frame = DecodedFrame::from(frame);
            frames_tx.send(frame).ok();
        }) as Box<dyn FnMut(_)>);

        let init = web_sys::VideoDecoderInit::new(
            on_error.as_ref().unchecked_ref(),
            on_frame.as_ref().unchecked_ref(),
        );
        let inner = web_sys::VideoDecoder::new(&init)?;

        Ok(Self {
            inner,
            on_error,
            on_frame,
            frames: frames_rx,
            closed: closed_rx,
        })
    }

    pub fn decode(&self, frame: EncodedFrame) -> Result<(), Error> {
        let chunk_type = match frame.keyframe {
            true => web_sys::EncodedVideoChunkType::Key,
            false => web_sys::EncodedVideoChunkType::Delta,
        };

        let chunk = web_sys::EncodedVideoChunkInit::new(
            &js_sys::Uint8Array::from(frame.data.as_slice()),
            frame.timestamp,
            chunk_type,
        );

        let chunk = web_sys::EncodedVideoChunk::new(&chunk)?;
        self.inner.decode(&chunk);

        Ok(())
    }

    pub fn configure(&self, config: &DecoderConfig) -> Result<(), Error> {
        self.inner.configure(&config.into());
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), Error> {
        wasm_bindgen_futures::JsFuture::from(self.inner.flush()).await?;
        Ok(())
    }

    pub async fn is_supported(config: &DecoderConfig) -> Result<bool, Error> {
        let res = wasm_bindgen_futures::JsFuture::from(web_sys::VideoDecoder::is_config_supported(
            &config.into(),
        ))
        .await?;

        let supported = js_sys::Reflect::get(&res, &JsValue::from_str("supported"))
            .unwrap()
            .as_bool()
            .unwrap();

        Ok(supported)
    }

    pub fn reset(&self) {
        self.inner.reset();
    }

    pub fn queue_size(&self) -> u32 {
        self.inner.decode_queue_size()
    }

    pub async fn decoded(&mut self) -> Result<DecodedFrame, Error> {
        tokio::select! {
            biased;
            Some(frame) = self.frames.recv() => Ok(frame),
            Ok(()) = self.closed.changed() => Err(self.closed.borrow().clone().err().unwrap()),
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        self.inner.close();
    }
}

pub struct DecoderConfig {
    codec: String,

    coded_dimensions: Option<(u32, u32)>,
    color_space: Option<ColorSpaceConfig>,
    display_dimensions: Option<(u32, u32)>,
    description: Option<Vec<u8>>,
    hardware_acceleration: Option<bool>,
    latency_optimized: bool,
}

impl DecoderConfig {
    pub fn new<T: Into<String>>(codec: T) -> Self {
        Self {
            codec: codec.into(),
            coded_dimensions: None,
            color_space: None,
            display_dimensions: None,
            description: None,
            hardware_acceleration: None,
            latency_optimized: false,
        }
    }

    pub fn coded_dimensions(mut self, width: u32, height: u32) -> Self {
        self.coded_dimensions = Some((width, height));
        self
    }

    pub fn display_dimensions(mut self, width: u32, height: u32) -> Self {
        self.display_dimensions = Some((width, height));
        self
    }

    pub fn description(mut self, description: Vec<u8>) -> Self {
        self.description = Some(description);
        self
    }

    pub fn color_space(mut self, color_space: ColorSpaceConfig) -> Self {
        self.color_space = Some(color_space);
        self
    }

    pub fn hardware_acceleration(mut self, preferred: bool) -> Self {
        self.hardware_acceleration = Some(preferred);
        self
    }

    pub fn latency_optimized(mut self) -> Self {
        self.latency_optimized = true;
        self
    }
}

impl From<&DecoderConfig> for web_sys::VideoDecoderConfig {
    fn from(this: &DecoderConfig) -> Self {
        let mut config = web_sys::VideoDecoderConfig::new(&this.codec);

        if let Some((width, height)) = this.coded_dimensions {
            config.coded_width(width);
            config.coded_height(height);
        }

        if let Some((width, height)) = this.display_dimensions {
            config.display_aspect_height(height);
            config.display_aspect_width(width);
        }

        if let Some(description) = &this.description {
            config.description(&js_sys::Uint8Array::from(description.as_slice()));
        }

        if let Some(color_space) = &this.color_space {
            config.color_space(&color_space.into());
        }

        if let Some(preferred) = this.hardware_acceleration {
            config.hardware_acceleration(match preferred {
                true => web_sys::HardwareAcceleration::PreferHardware,
                false => web_sys::HardwareAcceleration::PreferSoftware,
            });
        }

        if this.latency_optimized {
            config.optimize_for_latency(true);
        }

        config
    }
}
