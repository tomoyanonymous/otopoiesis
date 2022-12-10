use crate::audio::{Component, PlaybackInfo};
use crate::data;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Model {
    param: data::SharedVec<Arc<data::Region>>,
    channels: u64,
    regions: Vec<super::region::Model>,
}

impl Model {
    pub fn new(param: data::SharedVec<Arc<data::Region>>, channels: u64) -> Self {
        let regions = Self::get_new_regions(&param, channels);

        Self {
            param: Arc::clone(&param),
            channels,
            regions,
        }
    }
    fn get_new_regions(
        param: &data::SharedVec<Arc<data::Region>>,
        channels: u64,
    ) -> Vec<super::region::Model> {
        param
            .lock()
            .unwrap()
            .iter()
            .map(|region| super::region::Model::new(Arc::clone(&region), channels))
            .collect::<Vec<_>>()
    }
    fn renew_regions(&mut self, info: &PlaybackInfo) {
        //fetch update.

        let channels = info.channels;
        #[cfg(not(target_arch = "wasm32"))]
        let res = {
            let handles = self
                .param
                .lock()
                .unwrap()
                .iter()
                .map(|region| {
                    //temporary moves value to
                    let model = super::region::Model::new(Arc::clone(&region), channels);
                    super::region::render_region_offline_async(model, info)
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(move |h| h.join().expect("hoge"))
                .collect::<Vec<super::region::Model>>()
        };
        #[cfg(target_arch = "wasm32")]
        let res = self
            .param
            .lock()
            .unwrap()
            .iter()
            .map(|region| {
                let mut model = super::region::Model::new(Arc::clone(&region), channels);
                model.render_offline(info);
                model
            })
            .collect::<Vec<_>>();

        self.regions = res;
    }
}

impl Component for Model {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }
    fn prepare_play(&mut self, info: &PlaybackInfo) {
        self.renew_regions(info);
    }
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        //後に入ってるリージョンで基本は上書きする
        //channel is tekitou
        output.fill(0.0);
        for (count, out_per_channel) in output.chunks_mut(2 as usize).enumerate() {
            let now = (info.current_time + count) as u64;
            out_per_channel.iter_mut().enumerate().for_each(|(ch, s)| {
                //順にリージョンを読んでいくので、重なってる場合は後の要素のやつが上書きする形になる
                for region in self.regions.iter() {
                    if region.params.range.contains(now) {
                        let read_point = ((now - region.params.range.start()) * 2) as usize;
                        let out = region.interleaved_samples_cache[read_point + ch];
                        *s = out;
                    }
                }
            });
        }
    }
}
