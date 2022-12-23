use crate::audio::{Component, PlaybackInfo};
use crate::data;

#[derive(Debug)]
pub struct Model {
    param: Vec<data::Region>,
    channels: u64,
    regions: Vec<super::region::Model>,
}

impl Model {
    pub fn new(param: Vec<data::Region>, channels: u64) -> Self {
        let regions = Self::get_new_regions(&param, channels);

        Self {
            param,
            channels,
            regions,
        }
    }
    fn get_new_regions(
        param: &[data::Region],
        channels: u64,
    ) -> Vec<super::region::Model> {
        param
            .iter()
            .map(|region| super::region::Model::new(region.clone(), channels))
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
                    //temporary moves value to
                    let model = super::region::Model::new(region.clone(), channels);
                    super::region::render_region_offline_async(model, info)
                })
                .map(move |h| h.join().expect("hoge"))
                .collect::<Vec<super::region::Model>>()
        };
        #[cfg(target_arch = "wasm32")]
        let res = self
            .param
            .iter()
            .map(|region| {
                let mut model = super::region::Model::new(Arc::clone(&region), channels);
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
            let now = (info.current_time + count) as u64;
            out_per_channel.iter_mut().enumerate().for_each(|(ch, s)| {
                //順にリージョンを読んでいくので、重なってる場合は後の要素のやつが上書きする形になる
                for region in self.regions.iter() {
                    if region.params.range.contains(now) {
                        let read_point = ((now - region.params.range.start()) * chs) as usize;
                        // 再生中にRangeを変更すると範囲外アクセスの可能性はあるので対応
                        let out = region
                            .interleaved_samples_cache
                            .get(read_point + ch)
                            .unwrap_or(&0.0);
                        *s = *out;
                    }
                }
            });
        }
    }
}
