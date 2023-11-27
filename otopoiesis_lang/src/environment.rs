use super::*;

// Lexical Environment.
// It is doubley-linked for UI Generation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment {
    pub local: Vec<(String, Value)>,
    pub parent: Option<Arc<Self>>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bind(&mut self, key: &str, val: Value) {
        self.local.push((key.to_string(), val.clone()))
    }
    pub fn lookup(&self, key: &str) -> Option<Value> {
        self.local
            .iter()
            .find_map(|e| if e.0 == key { Some(e.1.clone()) } else { None })
            .or_else(|| self.parent.as_ref().and_then(|e| e.lookup(key)))
            .or_else(|| {
                builtin_fn::lookup_extfun(key)
                    .ok()
                    .map(|f| Value::ExtFunction(f))
            })
    }
}
impl Default for Environment {
    fn default() -> Self {
        Self {
            local: vec![],
            parent: None,
        }
    }
}
pub fn extend_env(env: Arc<Environment>) -> Environment {
    Environment {
        local: vec![],
        parent: Some(Arc::clone(&env)),
    }
}
