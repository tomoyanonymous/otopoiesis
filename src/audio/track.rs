use std::sync::Arc;

use crate::audio::{Component, PlaybackInfo};
use crate::data;
use crate::utils::AtomicRange;

use super::component::ScriptComponent;
use super::{RangedComponent, RangedComponentDyn};

#[derive(Debug)]
pub struct Model {
    param: Vec<data::Region>,
    _channels: u64,
    //actual regions contains sample caches
    regions: Vec<Box<dyn RangedComponent + Send + Sync>>,
}

impl Model {
    pub fn new(param: Vec<data::Region>, channels: u64) -> Self {
        let regions = Self::get_new_regions(&param, channels);

        Self {
            param,
            _channels: channels,
            regions,
        }
    }
    fn get_new_regions(
        param: &[data::Region],
        channels: u64,
    ) -> Vec<Box<dyn RangedComponent + Send + Sync>> {
        param
            .iter()
            .map(|region| match &region.content {
                data::Content::Generator(g) => Box::new(RangedComponentDyn::new(
                    Box::new(ScriptComponent::try_new(&g).expect("not an generator")),
                    AtomicRange::new(region.start.clone(), region.dur.clone()),
                ))
                    as Box<dyn RangedComponent + Send + Sync>,
                data::Content::Transformer(_, _) => todo!(),
            })
            .collect::<Vec<_>>()
    }
    fn renew_regions(&mut self, info: &PlaybackInfo) {
        //fetch update.

        let channels = info.channels;
        #[cfg(not(target_arch = "wasm32"))]
        let res = {
            self.param
                .iter()
                .map(|region| {
                    let model = match &region.content {
                        data::Content::Generator(g) => Box::new(RangedComponentDyn::new(
                            Box::new(ScriptComponent::try_new(&g).expect("not an generator")),
                            AtomicRange::new(region.start.clone(), region.dur.clone()),
                        ))
                            as Box<dyn RangedComponent + Send + Sync>,
                        data::Content::Transformer(_, _) => todo!(),
                    };
                    super::component::render_region_offline_async(model, info)
                })
                .map(|h| h.join().expect("failed to join threads"))
                .collect::<Vec<_>>()

            // self.param
            //     .iter()
            //     .map(|region| {
            //         //temporary moves value to
            //         let model = super::region::Model::new(region.clone(), channels);
            //         super::region::render_region_offline_async(model, info)
            //     })
            //     .map(move |h| h.join().expect("hoge"))
            //     .collect::<Vec<_>>()
        };
        #[cfg(target_arch = "wasm32")]
        let res = self
            .param
            .iter()
            .map(|region| {
                let mut model = super::region::Model::new(region.clone(), channels);
                model.render_offline(info.sample_rate, info.channels);
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
    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        //後に入ってるリージョンで基本は上書きする
        //channel is tekitou
        let chs = 2;
        output.fill(0.0);
        for (count, out_per_channel) in output.chunks_mut(chs as usize).enumerate() {
            let now = (info.current_time + count) as i64;
            let now_in_sec = now as f64 / info.sample_rate as f64;
            out_per_channel.iter_mut().enumerate().for_each(|(ch, s)| {
                //順にリージョンを読んでいくので、重なってる場合は後の要素のやつが上書きする形になる
                for region in self.regions.iter() {
                    let start_samp = (region.get_range().start() * info.sample_rate as f64) as i64;
                    if region.get_range().contains(&now_in_sec) {
                        let read_point = ((now - start_samp) * chs) as usize;
                        // 再生中にRangeを変更すると範囲外アクセスの可能性はあるので対応
                        let out = region
                            .get_sample_cache()
                            .get(read_point + ch)
                            .unwrap_or(&0.0);
                        *s = *out;
                    }
                }
            });
        }
    }
}
