use script::runtime::PlayInfo;

use crate::atomic::AtomicRange;
use crate::audio::{Component, PlaybackInfo};
use crate::data;

use super::component::ScriptComponent;
use super::{GenericRangedComponent, RangedComponent};

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
        _channels: u64,
    ) -> Vec<Box<dyn RangedComponent + Send + Sync>> {
        param
            .iter()
            .map(|region| {
                Box::new(GenericRangedComponent::new(
                    Box::new(ScriptComponent::try_new(&region.content).expect("not an generator")),
                    AtomicRange::new(region.start.clone(), region.dur.clone()),
                )) as Box<dyn RangedComponent + Send + Sync>
            })
            .collect::<Vec<_>>()
    }
    fn renew_regions(&mut self, info: &PlaybackInfo) {
        //fetch update.
        #[cfg(not(target_arch = "wasm32"))]
        let res = {
            self.param
                .iter()
                .map(|region| {
                    let model =
                        crate::audio::region::Model::new(region.clone(), info.get_channels());
                    super::component::render_region_offline_async(model.content, info)
                })
                .map(|h| h.join().expect("failed to join threads"))
                .collect::<Vec<_>>()
        };
        #[cfg(target_arch = "wasm32")]
        let res = self
            .param
            .iter()
            .map(|region| {
                let mut model = super::region::Model::new(region.clone(), info.get_channels());
                model.render_offline(info.sample_rate, info.get_channels());
                model.content
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
            let now = (info.get_current_time_in_sample() + count as u64) as i64;
            let now_in_sec = now as f64 / info.get_samplerate();
            out_per_channel.iter_mut().enumerate().for_each(|(ch, s)| {
                //順にリージョンを読んでいくので、重なってる場合は後の要素のやつが上書きする形になる
                for region in self.regions.iter() {
                    let start_samp = (region.get_range().start() * info.get_samplerate()) as i64;
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
