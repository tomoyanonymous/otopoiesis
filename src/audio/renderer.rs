use crate::audio::{Component, PlaybackInfo};
use crate::data;
use crate::utils::atomic;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{self, Stream};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use std::sync::{Arc, Mutex};

pub trait RendererBase<E>
where
    E: Component + Send + Sync + 'static,
{
    fn get_host(&self) -> &cpal::Host;
    fn set_streams(
        &mut self,
        i: Option<Stream>,
        o: Option<Stream>,
        iconfig: Option<cpal::StreamConfig>,
        oconfig: Option<cpal::StreamConfig>,
    );
    fn get_instream(&self) -> &Option<Stream>;
    fn get_outstream(&self) -> &Option<Stream>;
    fn init(
        &mut self,

        sample_rate: Option<u32>,

        imodel: Arc<Mutex<InputModel>>,
        omodel: Arc<Mutex<OutputModel<E>>>,
    ) {
        let host = cpal::default_host();
        let idevice = host.default_input_device();
        let (iconfig, istream) = if let Some(device) = idevice.as_ref() {
            let iconfig_builder = device
                .supported_input_configs()
                .unwrap()
                .next()
                .expect("no supported config?!");
            let iconfig = if let Some(sr) = sample_rate {
                iconfig_builder
                    .with_sample_rate(cpal::SampleRate(sr))
                    .config()
            } else {
                iconfig_builder.with_max_sample_rate().config()
            };
            let c = iconfig.clone();
            let in_stream = device.build_input_stream(
                &iconfig,
                move |data: &[f32], _s: &cpal::InputCallbackInfo| {
                    pass_in(imodel.clone(), data, c.clone())
                },
                |_e| {},
            );
            let _ = in_stream.as_ref().map(|i| i.pause());
            // if let Err(e) = &in_stream {
            //     web_sys::console::log_1(&e.to_string().into())
            // }
            (Some(iconfig), in_stream.ok())
        } else {
            (None, None)
        };

        let odevice = host.default_output_device();
        let (oconfig, ostream) = if let Some(device) = odevice.as_ref() {
            let oconfig_builder = device
                .supported_output_configs()
                .unwrap()
                .next()
                .expect("no supported config?!");
            let oconfig = if let Some(sr) = sample_rate {
                oconfig_builder
                    .with_sample_rate(cpal::SampleRate(sr))
                    .config()
            } else {
                oconfig_builder.with_max_sample_rate().config()
            };
            let oc = oconfig.clone();
            let out_stream = device.build_output_stream(
                &oconfig,
                move |data: &mut [f32], _s: &cpal::OutputCallbackInfo| {
                    pass_out(omodel.clone(), data, oc.clone())
                },
                |_e| {},
            );
            // if let Err(e) = &out_stream {
            //     web_sys::console::log_1(&e.to_string().into())
            // }
            let _ = out_stream.as_ref().map(|o| o.pause());
            (Some(oconfig), out_stream.ok())
        } else {
            (None, None)
        };

        self.set_streams(istream, ostream, iconfig, oconfig);
    }
    fn prepare_play(&mut self);
    fn is_playing(&self) -> bool;
    fn get_samplerate(&self) -> u32;
    fn play(&mut self);
    fn pause(&mut self);
    fn play_audio(&mut self) {
        if let Some(is) = self.get_instream() {
            is.play().unwrap();
        }
        if let Some(os) = self.get_outstream() {
            os.play().unwrap();
        }
    }
    fn pause_audio(&mut self) {
        if self.is_playing() {
            if let Some(is) = self.get_instream() {
                is.pause().unwrap();
            }
            if let Some(os) = self.get_outstream() {
                os.pause().unwrap();
            }
        }
    }
    fn toggle_play(&mut self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.play();
        }
    }
    fn get_shared_current_time_in_sample(&self) -> Arc<atomic::U64>;
    fn get_current_time_in_sample(&self) -> u64;
    fn get_current_time(&self) -> std::time::Duration {
        let now = self.get_current_time_in_sample();
        let now_f64 = self.get_outstream().as_ref().map_or(0., |os| {
            let sr = self.get_samplerate();
            now as f64 / sr as f64
        });
        std::time::Duration::from_secs_f64(now_f64)
    }
}

pub struct InputModel {
    pub producer: HeapProducer<f32>,
}

pub struct OutputModel<E: Component + Sync + Send> {
    pub consumer: HeapConsumer<f32>,
    pub internal_buf: Vec<f32>,
    pub effector: E,
    pub current_time: Arc<atomic::U64>,
}

fn pass_in(model: Arc<Mutex<InputModel>>, buffer: &[f32], info: cpal::StreamConfig) {
    if let Ok(mut m) = model.try_lock() {
        let _num = m.producer.push_slice(buffer);
    }
}

fn pass_out(
    model: Arc<Mutex<OutputModel<impl Component + Sync + Send>>>,
    buffer: &mut [f32],
    info: cpal::StreamConfig,
) {
    //assume input channels and output channels are the same
    if let Ok(mut model) = model.try_lock() {
        let len = buffer.len();
        let frame_per_buffer = len as u64 / info.channels as u64;
        let t = model.current_time.load();
        // let buf = &mut model.internal_buf.as_mut_slice()[0..len];
        let mut buf = vec![0.0; len];
        let _num = model.consumer.pop_slice(&mut buf);

        let info = PlaybackInfo {
            sample_rate: info.sample_rate.0.into(),
            current_time: t as usize,
            channels: info.channels as u64,
            frame_per_buffer,
        };
        // todo:if  channels are different?
        model.effector.render(&buf, buffer, &info);
        model.current_time.store(t + frame_per_buffer as u64);
    }
}

pub struct Renderer<E>
where
    E: Component + Send + Sync + 'static,
{
    pub host: cpal::Host,
    transport: Arc<data::Transport>,
    istream: Option<Stream>,
    ostream: Option<Stream>,
    imodel: Arc<Mutex<InputModel>>,
    omodel: Arc<Mutex<OutputModel<E>>>,
    iconfig: Option<cpal::StreamConfig>,
    oconfig: Option<cpal::StreamConfig>,
}

impl<E> RendererBase<E> for Renderer<E>
where
    E: Component + Send + Sync + 'static,
{
    fn get_host(&self) -> &cpal::Host {
        &self.host
    }
    fn set_streams(
        &mut self,
        i: Option<Stream>,
        o: Option<Stream>,
        iconfig: Option<cpal::StreamConfig>,
        oconfig: Option<cpal::StreamConfig>,
    ) {
        self.istream = i;
        self.ostream = o;
        self.iconfig = iconfig;
        self.oconfig = oconfig;
    }

    fn get_instream(&self) -> &Option<Stream> {
        &self.istream
    }
    fn get_outstream(&self) -> &Option<Stream> {
        &self.ostream
    }
    fn is_playing(&self) -> bool {
        self.transport.is_playing.load()
    }
    fn get_samplerate(&self) -> u32 {
        self.oconfig.as_ref().unwrap().sample_rate.0
    }

    fn play(&mut self) {
        self.play_audio();
        self.transport.is_playing.store(true);
    }

    fn pause(&mut self) {
        self.pause_audio();
        self.transport.is_playing.store(false);
    }

    fn get_shared_current_time_in_sample(&self) -> Arc<atomic::U64> {
        self.transport.time.clone()
    }

    fn get_current_time_in_sample(&self) -> u64 {
        self.transport.time.load()
    }

    fn prepare_play(&mut self) {
        if let Ok(_model) = self.imodel.lock() {
            //do nothing
        }
        let config = self.oconfig.as_ref().unwrap();
        let buffer_size = match config.buffer_size {
            cpal::BufferSize::Default => 512,
            cpal::BufferSize::Fixed(s) => s,
        };
        if let Ok(mut model) = self.omodel.lock() {
            let info = PlaybackInfo {
                sample_rate: config.sample_rate.0.into(),
                current_time: 0,
                frame_per_buffer: buffer_size as u64,
                channels: config.channels as u64,
            };
            model.effector.prepare_play(&info);
        }
    }
}

impl<E> Renderer<E>
where
    E: Component + Send + Sync + 'static,
{
    pub fn new(
        effect: E,
        sample_rate: Option<u32>,
        buffer_size: Option<usize>,
        transport: Arc<data::Transport>,
    ) -> Self {
        let latency_samples = buffer_size.unwrap_or(1024);
        let ring_buffer = HeapRb::<f32>::new(latency_samples * 4); // Add some latency
        let (producer, consumer) = ring_buffer.split();
        let mut res = Self {
            host: cpal::default_host(),
            transport: transport.clone(),
            istream: None,
            ostream: None,
            imodel: Arc::new(Mutex::new(InputModel { producer })),
            omodel: Arc::new(Mutex::new(OutputModel::<E> {
                consumer,
                internal_buf: vec![0.0; latency_samples],
                effector: effect,
                current_time: Arc::clone(&transport.time),
            })),
            iconfig: None,
            oconfig: None,
        };
        res.init(sample_rate, res.imodel.clone(), res.omodel.clone());
        res
    }
    pub fn rewind(&mut self) {
        self.get_shared_current_time_in_sample().store(0)
    }
}

pub fn create_renderer<E>(
    effect: E,
    sample_rate: Option<u32>,
    buffer_size: Option<usize>,
    transport: Arc<data::Transport>,
) -> Renderer<E>
where
    E: Component + Send + Sync + 'static,
{
    Renderer::<E>::new(effect, sample_rate, buffer_size, transport)
}
