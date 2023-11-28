
use crate::data;
use crate::gui;
use crate::parameter::{Parameter, RangedNumeric};

pub(crate) const BAR_WIDTH: f32 = 3.0;
use crate::script::ui::eval_ui_val;

mod region_handle;
// pub mod regionfilter;
// use self::regionfilter::fadeinout::FadeInOut;
// use self::regionfilter::replicate::Replicate;
// use self::regionfilter::RegionFilterState;
// use self::regionfilter::{fadeinout, replicate};
use super::generator::waveform::{State as WaveFormState, WaveForm};
use region_handle::{HandleMode, UiBar, UiBarState};

// pub enum ContentModel {
//     RegionFilter(regionfilter::RegionFilterState),
//     Generator(script::Value, WaveFormState),
// }

pub struct State {
    pub label: String,
    waveform: WaveFormState,
    range_handles: [UiBarState; 2],
    #[allow(dead_code)]
    is_interactive: bool,
}

impl State {
    pub fn renew_waveform(param: &data::Region) -> WaveFormState {
        let mut model = crate::audio::region::Model::new(param.clone(),2);
        model.render_offline(44100, 2);

        WaveFormState::new(
            model.content.get_sample_cache(),
            model.content.get_output_channels() as usize,
        )
    }
    pub fn new(params: &data::Region, labeltext: impl ToString, is_interactive: bool) -> Self {
        let waveform = Self::renew_waveform(&params);
        let handle_left = UiBarState::new(0.0..=params.dur.get().into());
        let handle_right = UiBarState::new(params.dur.get().into()..=f64::INFINITY);
        let range_handles = [handle_left, handle_right];
        Self {
            label: labeltext.to_string(),
            waveform,
            range_handles,
            is_interactive,
        }
    }
}

pub struct Model<'a> {
    pub params: &'a data::Region,
    pub state: &'a mut State,
}

impl<'a> Model<'a> {
    pub fn new(params: &'a data::Region, state: &'a mut State) -> Self {
        Self { params, state }
    }
    pub fn get_current_amp(&self) -> f32 {
        // self.osc_params.amp.get().abs()
        1.0
    }
    fn interact_main(&mut self, main: &mut egui::Response) {
        if main.dragged() {
            let offset = main.drag_delta().x as f64 / gui::PIXELS_PER_SEC_DEFAULT as f64;
            let start = self.params.start.get() as f64;
            self.params.start.set((start + offset) as f32);
            *main = main.clone().on_hover_cursor(egui::CursorIcon::Grabbing)
        }
    }
}

impl<'a> std::hash::Hash for Model<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.label.hash(state)
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let height = gui::TRACK_HEIGHT + 30.0;

        let start = self.params.start.get();
        let end = start + self.params.dur.get();
        let max_end = (end + self.params.dur.get_range().end()) as f64;

        //for debug
        // let rect = ui.available_rect_before_wrap();
        // ui.painter().rect_filled(rect, 0.0, egui::Color32::BLUE);

        ui.spacing_mut().item_spacing = egui::vec2(0., 0.);

        ui.horizontal_top(|ui| {
            let bar_size = egui::vec2(BAR_WIDTH, height);
            let mut start = self.params.start.get() as f64;
            let mut end = self.params.dur.get() as f64 + start;
            let mut handle_start = UiBar::new(
                &mut start,
                &mut self.state.range_handles[0],
                HandleMode::Start,
            );

            handle_start.set_limit(0.0..=*self.params.getrange().end() as f64);
            let startui = ui.add_sized(bar_size, handle_start);

            let wave_ui = WaveForm::new(&mut self.state.waveform, &44100.);
            let mut main = ui
                .add(wave_ui)
                .on_hover_cursor(egui::CursorIcon::Grab);
 
            let mut handle_end =
                UiBar::new(&mut end, &mut self.state.range_handles[1], HandleMode::End);
            handle_end.set_limit(*self.params.getrange().start()..=max_end);
            let endui = ui.add_sized(bar_size, handle_end);
            if startui.union(endui).drag_released() {
                self.state.waveform = State::renew_waveform(self.params);
            }

            if self.state.is_interactive {
                self.params.start.set(start as f32);
                self.params.dur.set((end - start) as f32);
                self.interact_main(&mut main);
            }
            let menu_rect1 = main.rect.right_bottom();
            let menu_rect2 = menu_rect1- egui::vec2(20.,20.);
            let menu_rect = egui::Rect::from_two_pos(menu_rect1,menu_rect2);
            ui.allocate_ui_at_rect(menu_rect, |ui|{
                ui.push_id(ui.next_auto_id(), |ui| {
                    egui::menu::menu_button(ui, "...", |ui| {
                        eval_ui_val(&self.params.content, ui).response
                    });
                });
            });
            main
        }).inner
    }
}
