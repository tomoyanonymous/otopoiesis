use crate::data;
use crate::data::Region;
use crate::gui;
use crate::parameter::Parameter;
use crate::parameter::RangedNumeric;
use crate::script;
use crate::script::Expr;
use crate::script::Value;
use crate::utils::atomic::SimpleAtomic;
use crate::utils::AtomicRange;
mod region_handle;
pub mod regionfilter;
use region_handle::{HandleMode, UiBar, UiBarState};

use self::regionfilter::fadeinout::FadeInOut;
use self::regionfilter::replicate::Replicate;
use self::regionfilter::RegionFilterState;
use self::regionfilter::{fadeinout, replicate};

pub enum ContentModel {
    RegionFilter(regionfilter::RegionFilterState),
    Generator(script::Value, super::generator::State),
}

pub struct State {
    pub label: String,
    content: ContentModel,
    range_handles: [UiBarState; 2],
    #[allow(dead_code)]
    is_interactive: bool,
}

impl State {
    pub fn new(params: &data::Region, labeltext: impl ToString, is_interactive: bool) -> Self {
        let handle_left = UiBarState::new(0.0..=params.dur.get().into());
        let handle_right = UiBarState::new(params.dur.get().into()..=f64::INFINITY);
        let content = match &params.content {
            data::Content::Generator(param) => {
                ContentModel::Generator(param.clone(), super::generator::State::new())
            }
            data::Content::Transformer(filter, origin) => {
                ContentModel::RegionFilter(match filter {
                    data::RegionFilter::Gain => todo!(),
                    data::RegionFilter::Reverse => todo!(),
                    data::RegionFilter::FadeInOut(_p) => {
                        regionfilter::RegionFilterState::FadeInOut(fadeinout::State::new(
                            origin,
                            origin.start.clone(),
                            origin.dur.clone(),
                        ))
                    }
                    data::RegionFilter::Replicate(p) => regionfilter::RegionFilterState::Replicate(
                        replicate::State::new(origin.as_ref(), p.count.load() as u64),
                    ),
                    data::RegionFilter::Script(v) => {
                        let (rg, _time_in, _time_out) = match v {
                            Value::Closure(
                                _ids,
                                _env,
                                box Expr::App(box Expr::Var(fname), args),
                            ) => match (fname.as_str(), args.as_slice()) {
                                (
                                    "apply_fade_in_out",
                                    [Expr::Literal(region), Expr::Literal(Value::Number(time_in)), Expr::Literal(Value::Number(time_out))],
                                ) => (
                                    Region::try_from(region).expect("not a function"),
                                    time_in,
                                    time_out,
                                ),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        };
                        regionfilter::RegionFilterState::FadeInOut(fadeinout::State::new(
                            &rg,
                            rg.start.clone(),
                            rg.dur.clone(),
                        ))
                    }
                })
            }
        };
        let range_handles = [handle_left, handle_right];
        Self {
            label: labeltext.to_string(),
            content,
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

        let bar_width = 5.;
        let start = self.params.start.get();
        let end = start + self.params.dur.get();
        let max_end = (end + self.params.dur.get_range().end()) as f64;

        //for debug
        // let rect = ui.available_rect_before_wrap();
        // ui.painter().rect_filled(rect, 0.0, egui::Color32::BLUE);

        ui.spacing_mut().item_spacing = egui::vec2(0., 0.);

        ui.horizontal_top(|ui| {
            let bar_size = egui::vec2(bar_width, height);
            let mut start = self.params.start.get() as f64;
            let mut end = self.params.dur.get() as f64 + start;

            let (mut main, is_interactive) = match (&self.params.content, &mut self.state.content) {
                (data::Content::Transformer(filter, origin), ContentModel::RegionFilter(state)) => {
                    match (filter, state) {
                        (data::RegionFilter::Gain, _) => todo!(),
                        (data::RegionFilter::FadeInOut(param), RegionFilterState::FadeInOut(s)) => {
                            //not needed to sync?
                            self.params.start.set(origin.start.get());
                            self.params.dur.set(origin.dur.get());
                            (
                                ui.add(regionfilter::RegionFilter::FadeInOut(FadeInOut::new(
                                    param,
                                    origin.as_ref(),
                                    s,
                                ))),
                                false,
                            )
                        }
                        (data::RegionFilter::Reverse, _) => todo!(),
                        (data::RegionFilter::Replicate(param), RegionFilterState::Replicate(s)) => {
                            (
                                ui.add(regionfilter::RegionFilter::Replicate(Replicate::new(
                                    param,
                                    origin.as_ref(),
                                    s,
                                ))),
                                false,
                            )
                        }
                        (_, _) => panic!(
                            "invalid combination of parameter and gui state in pattern matching "
                        ),
                    }
                }
                (data::Content::Generator(param), ContentModel::Generator(_genmodel, genstate)) => {
                    let main = ui
                        .group(|ui| {
                            ui.add_space(-bar_width);
                            let mut handle_start = UiBar::new(
                                &mut start,
                                &mut self.state.range_handles[0],
                                HandleMode::Start,
                            );
                            handle_start.set_limit(0.0..=*self.params.getrange().end() as f64);
                            let startui = ui.add_sized(bar_size, handle_start);
                            let gen = super::generator::Generator::new(
                                param,
                                AtomicRange::new(
                                    self.params.start.clone(),
                                    self.params.dur.clone(),
                                ),
                                genstate,
                            );
                            let main = ui.add(gen);
                            let mut handle_end = UiBar::new(
                                &mut end,
                                &mut self.state.range_handles[1],
                                HandleMode::End,
                            );
                            handle_end.set_limit(*self.params.getrange().start()..=max_end);
                            let endui = ui.add_sized(bar_size, handle_end);
                            if startui.union(endui).drag_released() {
                                let mut gen = super::generator::Generator::new(
                                    param,
                                    AtomicRange::new(
                                        self.params.start.clone(),
                                        self.params.dur.clone(),
                                    ),
                                    genstate,
                                );
                                gen.update_shape(&ui.ctx().style());
                            }
                            main
                        })
                        .inner;
                    (main, true)
                }
                _ => unreachable!(),
            };

            if is_interactive {
                self.params.start.set(start as f32);
                self.params.dur.set((end - start) as f32);
                self.interact_main(&mut main);
            }
        })
        .response
    }
}
