use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment<T>
where
    T: Clone,
{
    pub local: Vec<(Id, T)>,
    pub parent: Option<Arc<Self>>,
}

impl<T> Environment<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bind(&mut self, key: &Id, val: T) {
        self.local.push((key.clone(), val.clone()))
    }
    pub fn lookup(&self, key: &str) -> Option<&T> {
        self.local
            .iter()
            .find_map(|e| if &e.0 == key { Some(&e.1) } else { None })
            .or_else(|| self.parent.as_ref().and_then(|e| e.lookup(key)))
    }
}
impl<T: Clone> Default for Environment<T> {
    fn default() -> Self {
        Self {
            local: vec![],
            parent: None,
        }
    }
}
pub fn extend_env<T: Clone>(env: Arc<Environment<T>>) -> Environment<T> {
    Environment::<T> {
        local: vec![],
        parent: Some(Arc::clone(&env)),
    }
}
