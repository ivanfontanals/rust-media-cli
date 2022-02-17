use clap::{AppSettings, Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Parser)]
#[clap(name = "mediacli")]
#[clap(
    about = "A CLI to organize your media files",
    long_about = "Features: Media statistics and duplicate detections"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Gets media statistics, like number of files, size, etc...
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Info {
        #[clap(short)]
        folder: String,

        #[clap(short, long)]
        verbose: bool,
    },

    /// Detects binary duplictions with all your files inside your media folder
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Analyze {
        #[clap(short)]
        folder: String,

        #[clap(short, long)]
        verbose: bool,

        /// Delete duplications?
        #[clap(short)]
        delete: bool,
        /// Type can be: all, images, video
        #[clap(short('t'), name = "type", default_value = "images")]
        media_type: String,
    },
}
/*
file_path, hash, hash_1, etc....,size, nam
*/
