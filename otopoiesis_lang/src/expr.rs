use crate::runtime::PlayInfo;

use super::{value::Param, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Array(Vec<Expr>),
    Var(Id),
    Let(Id, Box<Expr>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>), //currently only single argument
    Lambda(Vec<Param>, Box<Expr>),
    //track and region is an alias to closure
    Track(Box<Expr>),
    Region(Box<Expr>, Box<Expr>, Box<Expr>, String), //start,dur,content
}

#[derive(Debug, Clone)]
pub enum EvalError {
    TypeMismatch(String),
    NotFound,
    InvalidNumArgs(usize, usize), //expected,actual
    InvalidConversion,
    NotInPlayMode,
    NoAppRuntime,
}

impl Expr {
    pub fn eval(
        &self,
        env: Arc<Environment>,
        play_info: &Option<&Box<dyn PlayInfo+Send+Sync>>,
    ) -> Result<Value, EvalError> {
        match self {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Array(vec) => {
                let v = vec
                    .iter()
                    .map(|e| e.eval(env.clone(), play_info))
                    .try_collect()?;
                Ok(Value::Array(
                    v,
                    Type::Array(Type::Unknown.into(), vec.len() as u64),
                ))
            }
            Expr::Var(v) => env.lookup(v).ok_or(EvalError::NotFound),
            Expr::Lambda(ids, body) => Ok(Value::Closure(ids.clone(), env.clone(), body.clone())),
            Expr::Let(id, body, then) => {
                let mut newenv = extend_env(env.clone());

                let body_v = body.eval(env, play_info)?;
                newenv.bind(id, body_v);

                then.eval(Arc::new(newenv), play_info)
            }
            Expr::App(fe, args) => {
                let f = fe.eval(env.clone(), play_info)?;
                let mut arg_res = vec![];
                for a in args.iter() {
                    match a.eval(env.clone(), play_info) {
                        Ok(res) => {
                            arg_res.push(res);
                        }
                        Err(e) => return Err(e),
                    }
                }
                match f {
                    Value::Function(_ids, _body) => {
                        todo!()
                    }
                    Value::Closure(ids, env, body) => {
                        let mut newenv = extend_env(env);
                        ids.iter().zip(arg_res.iter()).for_each(|(id, a)| {
                            newenv.bind(&id.get_label(), a.clone());
                        });
                        body.eval(Arc::new(newenv), play_info)
                    }
                    Value::ExtFunction(f) => f.0.exec(&env, play_info, &arg_res),
                    _ => Err(EvalError::TypeMismatch("Not a Function".into())),
                }
            }
            Expr::Track(content) => Ok(Value::Track(env.clone(), content.clone(), Type::Unknown)),
            Expr::Region(start, dur, content, label) => Ok(Value::Region(
                env.clone(),
                start.clone(),
                dur.clone(),
                content.clone(),
                label.clone(),
                Type::Unknown,
            )),
        }
    }
}
