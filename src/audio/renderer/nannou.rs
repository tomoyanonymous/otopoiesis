pub use super::RendererBase;
pub use crate::audio::Component;
use nannou_audio;

pub struct Renderer {
    pub host: nannou_audio::Host,
}

impl<E> super::RendererBase<E> for Renderer
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &nannou_audio::Host {
        &self.host
    }
}