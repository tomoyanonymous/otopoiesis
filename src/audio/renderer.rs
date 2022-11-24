use crate::audio::{Component, PlaybackInfo};
use nannou_audio;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use std::ops::DerefMut;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

pub trait RendererBase<E>
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &nannou_audio::Host;
    fn set_streams(
        &mut self,
        i: Option<nannou_audio::Stream<InputModel>>,
        o: Option<nannou_audio::Stream<OutputModel<E>>>,
    );
    fn get_instream(&self) -> &Option<nannou_audio::Stream<InputModel>>;
    fn get_outstream(&self) -> &Option<nannou_audio::Stream<OutputModel<E>>>;
    fn init(&mut self, effect: E, sample_rate: Option<u32>, buffersize: Option<usize>) {
        let host = self.get_host();
        let latency_samples = 1024;
        let ring_buffer = HeapRb::<f32>::new(latency_samples * 4); // Add some latency
        let (prod, cons) = ring_buffer.split();
        let in_model = InputModel { producer: prod };
        let out_model = OutputModel {
            consumer: cons,
            internal_buf: vec![0.0f32; latency_samples * 4],
            effector: effect,
            current_time: Arc::clone(&self.get_shared_current_time_in_sample()),
        };

        let mut in_stream_builder = host.new_input_stream(in_model).capture(pass_in).channels(2);

        let mut out_stream_builder = host
            .new_output_stream(out_model)
            .render(pass_out)
            .channels(2);
        if let Some(sr) = sample_rate {
            in_stream_builder = in_stream_builder.sample_rate(sr);
            out_stream_builder = out_stream_builder.sample_rate(sr);
        }
        if let Some(bufsize) = buffersize {
            in_stream_builder = in_stream_builder.frames_per_buffer(bufsize);
            out_stream_builder = out_stream_builder.frames_per_buffer(bufsize);
        }

        let in_stream = in_stream_builder.build().unwrap();
        let out_stream = out_stream_builder.build().unwrap();
        self.set_streams(Some(in_stream), Some(out_stream));
    }
    fn prepare_play(&mut self) {
        self.get_instream().as_ref().map(|i| {
            i.send(|_imodel| {
                //do nothing
            })
        });
        self.get_outstream().as_ref().map(|o| {
            let sr = o.cpal_config().sample_rate.0;
            o.send(
                move |omodel| {
                    omodel.effector.prepare_play(&PlaybackInfo {
                        sample_rate: sr,
                        current_time: 0,
                    })
                }, //do nothing
            )
        });
    }
    fn is_playing(&self) -> bool {
        self.get_instream()
            .as_ref()
            .map_or(false, |i| i.is_playing())
            || self
                .get_outstream()
                .as_ref()
                .map_or(false, |o| o.is_playing())
    }
    fn play(&mut self) {
        if let Some(is) = self.get_instream() {
            is.play().unwrap();
        }
        if let Some(os) = self.get_outstream() {
            os.play().unwrap();
        }
    }
    fn pause(&mut self) {
        if let Some(is) = self.get_instream() {
            if is.is_playing() {
                is.pause().unwrap();
            }
        }
        if let Some(os) = self.get_outstream() {
            if os.is_playing() {
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
    fn get_shared_current_time_in_sample(&self) -> Arc<AtomicU64>;
    fn get_current_time_in_sample(&self) -> u64;
    fn get_current_time(&self) -> std::time::Duration {
        let now = self.get_current_time_in_sample();
        let now_f64 = self.get_outstream().as_ref().map_or(0.,  |os| {
            let sr = os.cpal_config().sample_rate.0;
            now as f64 / sr as f64
        });
        std::time::Duration::from_secs_f64(now_f64)
    }
}

pub struct InputModel {
    pub producer: HeapProducer<f32>,
}

pub struct OutputModel<E: Component> {
    pub consumer: HeapConsumer<f32>,
    pub internal_buf: Vec<f32>,
    pub effector: E,
    pub current_time: Arc<AtomicU64>,
}

fn pass_in(model: &mut InputModel, buffer: &nannou_audio::Buffer) {
    let _num = model.producer.push_slice(buffer.as_ref());
    // println!("{:.2?}",buffer.as_ref());
}

fn pass_out(model: &mut OutputModel<impl Component>, buffer: &mut nannou_audio::Buffer) {
    //assume input channels and output channels are the same
    let len = buffer.channels() * buffer.len_frames();
    let buf = &mut model.internal_buf.as_mut_slice()[0..len];
    let _num = model.consumer.pop_slice(buf);
    let t = model.current_time.load(Ordering::Relaxed);
    let info = PlaybackInfo {
        sample_rate: buffer.sample_rate(),
        current_time: t as usize,
    };
    // todo:if  channels are different?
    model.effector.render(buf, buffer.deref_mut(), &info);
    model
        .current_time
        .store(t + buffer.len_frames() as u64, Ordering::Relaxed);
}

pub struct Renderer<E>
where
    E: Component + Send + 'static,
{
    pub host: nannou_audio::Host,
    current_time_in_sample: Arc<AtomicU64>,
    istream: Option<nannou_audio::Stream<InputModel>>,
    ostream: Option<nannou_audio::Stream<OutputModel<E>>>,
}

impl<E> RendererBase<E> for Renderer<E>
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &nannou_audio::Host {
        &self.host
    }
    fn set_streams(
        &mut self,
        i: Option<nannou_audio::Stream<InputModel>>,
        o: Option<nannou_audio::Stream<OutputModel<E>>>,
    ) {
        self.istream = i;
        self.ostream = o;
    }
    fn get_instream(&self) -> &Option<nannou_audio::Stream<InputModel>> {
        &self.istream
    }
    fn get_outstream(&self) -> &Option<nannou_audio::Stream<OutputModel<E>>> {
        &self.ostream
    }
    fn get_shared_current_time_in_sample(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.current_time_in_sample)
    }
    fn get_current_time_in_sample(&self) -> u64{
        self.current_time_in_sample.load(Ordering::Relaxed)
    }
}

impl<E> Renderer<E>
where
    E: Component + Send + 'static,
{
    pub fn new(effect: E, sample_rate: Option<u32>, buffer_size: Option<usize>) -> Self {
        let mut res = Self {
            host: nannou_audio::Host::default(),
            current_time_in_sample: Arc::new(AtomicU64::from(0)),
            istream: None,
            ostream: None,
        };
        res.init(effect, sample_rate, buffer_size);
        res
    }
    pub fn rewind(&mut self) {
        self.get_shared_current_time_in_sample()
            .store(0, Ordering::Relaxed)
    }
}

pub fn create_renderer<E>(
    effect: E,
    sample_rate: Option<u32>,
    buffer_size: Option<usize>,
) -> Renderer<E>
where
    E: Component + Send + 'static,
{
    Renderer::<E>::new(effect, sample_rate, buffer_size)
}
