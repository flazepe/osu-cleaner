use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    /// The path to your osu! songs directory.
    pub song_directory_path: PathBuf,

    /// Whether to delete background images. This will cause warnings from osu!
    #[arg(short = 'b', long = "backgrounds")]
    pub backgrounds: bool,

    /// Whether to delete all unneeded files, excluding backgrounds.
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Whether to delete storyboard files, including subdirectories inside the beatmapset folder
    #[arg(short = 't', long = "storyboards")]
    pub storyboards: bool,

    /// Whether to delete video files
    #[arg(short = 'v', long = "videos")]
    pub videos: bool,

    /// Whether to delete miscellaneous image files, like custom skins
    #[arg(short = 'i', long = "images")]
    pub images: bool,

    /// Whether to delete miscellaneous sound files, like custom hitsounds
    #[arg(short = 's', long = "sounds")]
    pub sounds: bool,

    /// Whether to perform deletions quietly
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Whether to enable debug mode. This won't delete your files
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
}
