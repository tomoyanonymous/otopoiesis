use nannou_audio;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use std::{sync::Arc, ops::DerefMut};
pub mod oscillator;
pub trait Component {
    fn get_input_channels(&self) -> usize;
    fn get_output_channels(&self) -> usize;
    fn render(&mut self, input: &[f32], output: &mut [f32]);
}

pub trait Renderer<E>
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &nannou_audio::Host;

    fn init(
        &mut self,
        effect: E,
    ) -> (
        nannou_audio::Stream<InputModel>,
        nannou_audio::Stream<OutputModel<E>>,
    ) {
        let (in_model, out_model) = init_models(effect);

        let in_stream = self
            .get_host()
            .new_input_stream(in_model)
            .capture(pass_in)
            .device_buffer_size(nannou_audio::BufferSize::Fixed(1024))
            .frames_per_buffer(256)
            .channels(2)
            .build()
            .unwrap();
        let out_stream = self
            .get_host()
            .new_output_stream(out_model)
            .render(pass_out)
            .device_buffer_size(nannou_audio::BufferSize::Fixed(1024))
            .frames_per_buffer(256)
            .channels(2)
            .build()
            .unwrap();
        in_stream.play().unwrap();

        out_stream.play().unwrap();
        println!("input");
        (in_stream, out_stream)
    }
}

pub struct InputModel {
    pub producer: HeapProducer<f32>,
}

pub struct OutputModel<E: Component> {
    pub consumer: HeapConsumer<f32>,
    pub internal_buf: Vec<f32>,
    pub effector: E,
}

pub fn init_models<E: Component>(effect: E) -> (InputModel, OutputModel<E>) {
    let latency_samples = 1024;
    let ring_buffer = HeapRb::<f32>::new(latency_samples * 4); // Add some latency
    let (prod, cons) = ring_buffer.split();
    let inmodel = InputModel { producer: prod };
    let outmodel = OutputModel {
        consumer: cons,
        internal_buf: vec![0.0f32; latency_samples * 4],
        effector: effect,
    };
    (inmodel, outmodel)
}

fn pass_in(model: &mut InputModel, buffer: &nannou_audio::Buffer) {
    let _num = model.producer.push_slice(buffer.as_ref());
}

fn pass_out<E: Component>(model: &mut OutputModel<E>, buffer: &mut nannou_audio::Buffer) {
    //assume input channels and output channels are the same
    let len = buffer.channels() * buffer.len_frames();
    let buf = &mut model.internal_buf.as_mut_slice()[0..len];
    let _num = model.consumer.pop_slice(buf);
    // todo:if  channels are different?
    model.effector.render(buf, buffer.deref_mut());
}

pub struct SimpleRenderer {
    pub host: nannou_audio::Host,
}

impl<E> Renderer<E> for SimpleRenderer
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &nannou_audio::Host {
        &self.host
    }
}
