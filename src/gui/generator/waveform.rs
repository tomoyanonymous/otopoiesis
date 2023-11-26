use crate::gui::PIXELS_PER_SEC_DEFAULT;
use std::{ops::RangeInclusive, slice::Chunks};
use crate::gui::region::BAR_WIDTH;

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

    data.enumerate()
        .map(|(i, range)| {
            let min = range.start();
            let max = range.end();
            mesh.colored_vertex(egui::pos2(x, min * 0.5 * height), color);
            mesh.colored_vertex(egui::pos2(x, max * 0.5 * height), color);
            x += width_per_sample * THUMBNAIL_REDUCTION_RATE as f32;
            i as u32 * 2
        })
        .collect::<Vec<_>>()
        .iter()
        .reduce(|id0, id2| {
            let id1 = id0 + 1;
            let id3 = id2 + 1;
            mesh.add_triangle(*id0, id1, *id2);
            mesh.add_triangle(id1, *id2, id3);
            id2
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
    pub fn new(data: &[f32], channels: usize) -> Self {
        //only mono
        let chunks = data.chunks(THUMBNAIL_REDUCTION_RATE * channels);
        let reduced_samples = chunks
            .into_iter()
            .map(|chunk| {
                let min = chunk
                    .into_iter()
                    .step_by(channels)
                    .fold(0.0f32, |a, b| a.min(*b));
                let max = chunk
                    .into_iter()
                    .step_by(channels)
                    .fold(0.0f32, |a, b| a.max(*b));
                let rms = calc_rms_average(chunk);

                (min..=max, rms)
            })
            .collect::<Vec<_>>();
        Self {
            num_samples: data.len(),
            reduced_samples,
        }
    }
    pub fn gen_peak_shape(&self, width_per_sample: f32, height: f32) -> egui::Shape {
        let color = egui::Color32::LIGHT_GRAY;
        let chunks = self.reduced_samples.iter().map(|(range, _)| range.clone());
        egui::Shape::Mesh(gen_mesh_from_minmax(
            chunks.into_iter(),
            width_per_sample,
            Y_DEFAULT,
            height,
            color,
        ))
    }
    pub fn gen_rms_shape(&self, width_per_sample: f32, height: f32) -> egui::Shape {
        let color = egui::Color32::LIGHT_GRAY;
        let chunks = self.reduced_samples.iter().map(|(_range, rms)| -rms..=*rms);
        egui::Shape::Mesh(gen_mesh_from_minmax(
            chunks.into_iter(),
            width_per_sample,
            Y_DEFAULT,
            height,
            color.linear_multiply(0.8),
        ))
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

        let visible_rect = ui
            .clip_rect()
            .intersect(available_rect)
            .translate(egui::vec2(BAR_WIDTH, 0.0));

        let response = ui.allocate_rect(visible_rect, egui::Sense::click_and_drag());
        let painter = ui.painter();
        //todo:when zoomed in, draw lines directly
        painter.rect_filled(visible_rect, 0.0, egui::Color32::DARK_GRAY);
        let translate_vec = visible_rect.left_center().to_vec2();
        let mut peak = egui::Shape::from(
            self.state
                .gen_peak_shape(width_per_sample, visible_rect.height()),
        );
        peak.translate(translate_vec);
        painter.add(peak);
        let mut rms = self
            .state
            .gen_rms_shape(width_per_sample, visible_rect.height());
        rms.translate(translate_vec);
        painter.add(rms);

        // painter.add(self.state.shape_rms);
        response
    }
}
