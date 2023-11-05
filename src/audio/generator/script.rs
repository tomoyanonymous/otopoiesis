use super::Component;
use crate::audio::{PlaybackInfo, RenderCtx};
use crate::data::expr::{self, EvalEnv, Value};

#[derive(Clone, Debug)]
pub struct Script(pub expr::Expr);

impl Script {
    pub fn eval(&self, env: &mut EvalEnv, sample_rate: u32) -> f64 {
        match self.0.eval(env) {
            Ok(Value::Real(v)) => v,
            _ => {
                eprintln!("generator eval is not real");
                0.0
            }
        }
    }
}

impl Component for Script {
    fn get_input_channels(&self) -> u64 {
        todo!()
    }

    fn get_output_channels(&self) -> u64 {
        todo!()
    }

    fn prepare_play(&mut self, info: &PlaybackInfo) {
        todo!()
    }

    fn render(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        info: &PlaybackInfo,
        ctx: &mut RenderCtx,
    ) {
        let env = &mut ctx.env;
        let mut now = match env.global.get("now").unwrap().clone() {
            Value::Int(v) => v,
            _ => {
                eprintln!("not integer");
                0
            }
        };

        output.iter_mut().for_each(|o| {
            *o = self.eval(env, info.sample_rate) as f32;
            now += 1;
            env.global.insert("now".into(), Value::Int(now));
        });
    }
}
