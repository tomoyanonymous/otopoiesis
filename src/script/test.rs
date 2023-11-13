#[test]
fn eval_expr() {
    use super::*;
    use crate::data::{GlobalSetting, LaunchArg, Transport};
    use data::AppModel;
    let mut app = AppModel::new(Transport::new(), GlobalSetting, LaunchArg::default());
    
}
