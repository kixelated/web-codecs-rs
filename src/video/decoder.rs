use bytes::Bytes;
use tokio::sync::{mpsc, watch};
use wasm_bindgen::prelude::*;

use super::VideoColorSpaceConfig;
use crate::{EncodedFrame, Error, VideoFrame};

pub fn video_decoder() -> (VideoDecoder, VideoDecoded) {
    let (frames_tx, frames_rx) = mpsc::unbounded_channel();
    let (closed_tx, closed_rx) = watch::channel(Ok(()));
    let closed_tx2 = closed_tx.clone();

    let on_error = Closure::wrap(Box::new(move |e: JsValue| {
        closed_tx.send_replace(Err(Error::from(e))).ok();
    }) as Box<dyn FnMut(_)>);

    let on_frame = Closure::wrap(Box::new(move |e: JsValue| {
        let frame: web_sys::VideoFrame = e.unchecked_into();
        let frame = VideoFrame::from(frame);

        if frames_tx.send(frame).is_err() {
            closed_tx2.send_replace(Err(Error::Dropped)).ok();
        }
    }) as Box<dyn FnMut(_)>);

    let init = web_sys::VideoDecoderInit::new(
        on_error.as_ref().unchecked_ref(),
        on_frame.as_ref().unchecked_ref(),
    );
    let inner = web_sys::VideoDecoder::new(&init).unwrap();

    let decoder = VideoDecoder {
        inner,
        on_error,
        on_frame,
    };

    let decoded = VideoDecoded {
        frames: frames_rx,
        closed: closed_rx,
    };

    (decoder, decoded)
}

pub struct VideoDecoder {
    inner: web_sys::VideoDecoder,

    // These are held to avoid dropping them.
    #[allow(dead_code)]
    on_error: Closure<dyn FnMut(JsValue)>,
    #[allow(dead_code)]
    on_frame: Closure<dyn FnMut(JsValue)>,
}

impl VideoDecoder {
    pub fn decode(&self, frame: EncodedFrame) -> Result<(), Error> {
        let chunk_type = match frame.keyframe {
            true => web_sys::EncodedVideoChunkType::Key,
            false => web_sys::EncodedVideoChunkType::Delta,
        };

        let chunk = web_sys::EncodedVideoChunkInit::new(
            &js_sys::Uint8Array::from(frame.payload.as_ref()),
            frame.timestamp,
            chunk_type,
        );

        let chunk = web_sys::EncodedVideoChunk::new(&chunk)?;
        self.inner.decode(&chunk)?;

        Ok(())
    }

    pub fn configure(&self, config: &VideoDecoderConfig) -> Result<(), Error> {
        self.inner.configure(&config.into())?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), Error> {
        wasm_bindgen_futures::JsFuture::from(self.inner.flush()).await?;
        Ok(())
    }

    pub async fn is_supported(config: &VideoDecoderConfig) -> Result<bool, Error> {
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

    pub fn reset(&self) -> Result<(), Error> {
        self.inner.reset()?;
        Ok(())
    }

    pub fn queue_size(&self) -> u32 {
        self.inner.decode_queue_size()
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        let _ = self.inner.close();
    }
}

pub struct VideoDecoded {
    frames: mpsc::UnboundedReceiver<VideoFrame>,
    closed: watch::Receiver<Result<(), Error>>,
}

impl VideoDecoded {
    pub async fn next(&mut self) -> Result<Option<VideoFrame>, Error> {
        tokio::select! {
            biased;
            frame = self.frames.recv() => Ok(frame),
            Ok(()) = self.closed.changed() => Err(self.closed.borrow().clone().err().unwrap()),
        }
    }
}

pub struct VideoDecoderConfig {
    codec: String,

    coded_dimensions: Option<(u32, u32)>,
    color_space: Option<VideoColorSpaceConfig>,
    display_dimensions: Option<(u32, u32)>,
    description: Option<Bytes>,
    hardware_acceleration: Option<bool>,
    latency_optimized: bool,
}

impl VideoDecoderConfig {
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

    pub fn description(mut self, description: Bytes) -> Self {
        self.description = Some(description);
        self
    }

    pub fn color_space(mut self, color_space: VideoColorSpaceConfig) -> Self {
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

impl From<&VideoDecoderConfig> for web_sys::VideoDecoderConfig {
    fn from(this: &VideoDecoderConfig) -> Self {
        let config = web_sys::VideoDecoderConfig::new(&this.codec);

        if let Some((width, height)) = this.coded_dimensions {
            config.set_coded_width(width);
            config.set_coded_height(height);
        }

        if let Some((width, height)) = this.display_dimensions {
            config.set_display_aspect_height(height);
            config.set_display_aspect_width(width);
        }

        if let Some(description) = &this.description {
            config.set_description(&js_sys::Uint8Array::from(description.as_ref()));
        }

        if let Some(color_space) = &this.color_space {
            config.set_color_space(&color_space.into());
        }

        if let Some(preferred) = this.hardware_acceleration {
            config.set_hardware_acceleration(match preferred {
                true => web_sys::HardwareAcceleration::PreferHardware,
                false => web_sys::HardwareAcceleration::PreferSoftware,
            });
        }

        if this.latency_optimized {
            config.set_optimize_for_latency(true);
        }

        config
    }
}
