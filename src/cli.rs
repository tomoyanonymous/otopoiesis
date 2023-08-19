use crate::data::LaunchArg;
pub use clap::Parser;

/// otopoiesis - constructive sound design environment
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of project file to open
    file: Option<String>,
    /// Path of project directory. Ignored when the file path is absolute
    #[arg(short, long)]
    project_root: Option<String>,
    /// Global Config Directory (default: ${HOME}/.otopoieis)
    #[arg(short, long)]
    config_dir: Option<String>,
    /// (currently not implemented) log infomation level (1:trace 2:info 3:warn 4:error 5:none)
    #[arg(short, long, default_value_t = 3)]
    log_level: u8,
}

impl From<Args> for LaunchArg {
    fn from(val: Args) -> Self {
        let arg = LaunchArg::default();
        LaunchArg {
            file: val.file.or(arg.file),
            project_root: val.project_root.or(arg.project_root),
            config_dir: val.config_dir.or(arg.config_dir),
            log_level: val.log_level,
        }
    }
}
