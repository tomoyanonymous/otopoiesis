// data format for project file. serialized to json with serde.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GlobalSetting;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub global_setting: GlobalSetting,
}
