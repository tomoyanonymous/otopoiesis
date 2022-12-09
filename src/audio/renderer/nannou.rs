pub use super::RendererBase;
pub use crate::audio::Component;
use cpal;

pub struct Renderer {
    pub host: nannou_audio::Host,
}

impl<E> super::RendererBase<E> for Renderer
where
    E: Component + Send + 'static,
{
    fn get_host(&self) -> &cpal::Host {
        &self.host
    }
}