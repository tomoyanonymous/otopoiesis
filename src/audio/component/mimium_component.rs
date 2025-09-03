use super::Component;
use mimium_lang::{
    ExecContext,
    interner::{Symbol, ToSymbol},
    runtime::vm::Machine,
};

pub struct MimiumComponent {
    vm: Machine,
    dsp_idx: Symbol,
}
unsafe impl Send for MimiumComponent {}
unsafe impl Sync for MimiumComponent {}
impl MimiumComponent {
    pub fn new(vm: Machine) -> Self {
        let dsp_idx = "dsp".to_symbol();
        Self { vm, dsp_idx }
    }
}

impl Component for MimiumComponent {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        1
    }

    fn prepare_play(&mut self, info: &crate::audio::PlaybackInfo) {
        self.vm.clear_stack();
        self.vm.clear_states();
        self.vm.execute_main();
    }

    fn render(&mut self, input: &[f32], output: &mut [f32], info: &crate::audio::PlaybackInfo) {
        for o in output.iter_mut() {
            let _ = self.vm.execute_entry(&self.dsp_idx);
            let res = Machine::get_as::<f64>(self.vm.get_stack(0));
            *o = res as f32;
        }
    }
}
