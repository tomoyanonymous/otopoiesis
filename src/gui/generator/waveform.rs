use crate::gui::PIXELS_PER_SEC_DEFAULT;
use std::{ops::RangeInclusive, slice::Chunks};

const THUMBNAIL_REDUCTION_RATE: usize = 256;
// const WIDTH_PER_SAMPLE_DEFAULT: f32 = 1.0;
const Y_DEFAULT: f32 = 10.0;
// const HEIGHT_DEFAULT: f32 = 1.0;

pub struct State {
    //sample size of original waveform
    num_samples: usize,
    reduced_samples: Vec<(RangeInclusive<f32>, f32)>, //min,max,rms
                                                      // shape_peak: Shape,
                                                      // shape_rms: Shape,
}
fn calc_rms_average(chunk: &[f32]) -> f32 {
    let len = chunk.len() as f32;
    chunk.iter().fold(0.0, |acc, s| acc + s * s / len).sqrt()
}

fn gen_mesh_from_minmax<D>(
    data: D,
    width_per_sample: f32,
    _y: f32,
    height: f32,
    color: egui::Color32,
) -> egui::Mesh
where
    D: std::iter::Iterator<Item = RangeInclusive<f32>>,
{
    let mut mesh = egui::Mesh::default();
    let mut x = 0.0;
    data.fold(0, |prev_top_id, range| {
        let prev_bottom_id = prev_top_id + 1;
        let min = range.start();
        let max = range.end();
        let bottom_id = prev_bottom_id + 1;
        let top_id = bottom_id + 1;

        mesh.colored_vertex(egui::pos2(x, min * height), color);
        mesh.colored_vertex(egui::pos2(x, max * height), color);
        mesh.add_triangle(prev_bottom_id, prev_top_id, top_id);
        mesh.add_triangle(top_id, prev_bottom_id, bottom_id);
        x += width_per_sample;

        top_id + 1
    });
    mesh
}

impl State {
    pub fn gen_shape_from_chunk<F: FnMut(&[f32]) -> RangeInclusive<f32>>(
        chunks: &Chunks<'_, f32>,
        width_per_sample: f32,
        y: f32,
        height: f32,
        color: egui::Color32,
        algorithm: F,
    ) -> impl Into<egui::Shape> {
        let minmaxs = chunks.clone().map(algorithm);
        let mesh_peak =
            gen_mesh_from_minmax(minmaxs.into_iter(), width_per_sample, y, height, color);
        mesh_peak
    }
    pub fn new(data: &[f32]) -> Self {
        //only mono
        let chunks = data.chunks(THUMBNAIL_REDUCTION_RATE);
        let reduced_samples = chunks
            .into_iter()
            .map(|chunk| {
                let min = chunk.into_iter().fold(0.0f32, |a, b| a.min(*b));
                let max = chunk.into_iter().fold(0.0f32, |a, b| a.max(*b));
                let rms = calc_rms_average(chunk);
                (min..=max, rms)
            })
            .collect::<Vec<_>>();
        Self {
            num_samples: data.len(),
            reduced_samples,
        }
    }
    pub fn gen_peak_shape(&self, width_per_sample: f32, height: f32) -> impl Into<egui::Shape> {
        let color = egui::Color32::WHITE;
        let chunks = self.reduced_samples.iter().map(|(range, _)| range.clone());
        gen_mesh_from_minmax(
            chunks.into_iter(),
            width_per_sample,
            Y_DEFAULT,
            height,
            color.linear_multiply(0.8),
        )
    }
    pub fn gen_rms_shape(&self, width_per_sample: f32, height: f32) -> impl Into<egui::Shape> {
        let color = egui::Color32::WHITE;
        let chunks = self.reduced_samples.iter().map(|(_range, rms)| -rms..=*rms);
        gen_mesh_from_minmax(
            chunks.into_iter(),
            width_per_sample,
            Y_DEFAULT,
            height,
            color.linear_multiply(0.8),
        )
    }
}

pub struct WaveForm<'a> {
    state: &'a mut State,
    sample_rate: &'a f32,
}
impl<'a> WaveForm<'a> {
    pub fn new(state: &'a mut State, sample_rate: &'a f32) -> Self {
        Self { state, sample_rate }
    }
}

/// many part of code is reffering to https://codeberg.org/xlambein/futile/src/branch/main/paw/src/widgets/track/audio_clip.rs
impl<'a> egui::Widget for WaveForm<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width_per_sample = PIXELS_PER_SEC_DEFAULT / *self.sample_rate;
        // let width = self.state.num_samples as f32 * width_per_sample;
        // The waveform will take up all the available space, mapping
        // `clip.start` to `ui.max_rect().left()` and `clip.end` to
        // `ui.max_rect().right()`
        let available_rect = ui.max_rect();
        // However, we only wanna draw whatever's visible
        let visible_rect = ui.clip_rect().intersect(available_rect);

        let response = ui.allocate_rect(available_rect, egui::Sense::hover());
        let painter = ui.painter();
        //todo:when zoomed in, draw lines directly
        painter.rect_filled(visible_rect, 0.0, egui::Color32::BLACK);
        painter.add(
            self.state
                .gen_peak_shape(width_per_sample, visible_rect.height()),
        );
        painter.add(
            self.state
                .gen_rms_shape(width_per_sample, visible_rect.height()),
        );

        // painter.add(self.state.shape_rms);
        response
    }
}
