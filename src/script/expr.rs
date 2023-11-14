use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Var(Id),
    Let(Id, Box<Expr>, Box<Expr>),
    Lambda(Vec<Id>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>), //currently only single argument
}

pub enum EvalError {
    TypeMismatch(String),
    NotFound,
    InvalidNumArgs(usize, usize), //expected,actual
}

impl Expr {
    pub fn eval(
        &self,
        env: Arc<Environment<Value>>,
        app: &mut Option<&mut data::AppModel>,
    ) -> Result<Value, EvalError> {
        match self {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Var(v) => env.lookup(v).ok_or(EvalError::NotFound).cloned(),
            Expr::Lambda(ids, body) => Ok(Value::Closure(ids.clone(), env.clone(), body.clone())),
            Expr::Let(id, body, then) => {
                let mut newenv = extend_env(env.clone());

                let body_v = body.eval(env, app)?;
                newenv.bind(id, body_v);

                then.eval(Arc::new(newenv), app)
            }
            Expr::App(fe, args) => {
                let f = fe.eval(env.clone(), app)?;
                let mut arg_res = vec![];
                for a in args.iter() {
                    match a.eval(env.clone(), app) {
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
                            newenv.bind(id, a.clone());
                        });
                        body.eval(Arc::new(newenv), app)
                    }
                    Value::ExtFunction(fname) => {
                        if app.is_some() {
                            let mut a = app.as_ref().unwrap();
                            let f = a.get_builtin_fn(&fname).ok_or(EvalError::NotFound)?.clone();
                            f.0.exec(app, &arg_res)
                        } else {
                            Err(EvalError::NotFound)
                        }
                    }
                    _ => Err(EvalError::TypeMismatch("Not a Function".into())),
                }
            }
        }
    }
}
