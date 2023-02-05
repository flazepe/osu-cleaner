use crate::args::Args;
use anyhow::{Context, Result};
use clap::Parser;
use std::{
    fs::{read_dir, read_to_string, remove_dir_all, remove_file, File},
    path::PathBuf,
};

const VIDEO_EXTS: [&str; 5] = ["avi", "flv", "m4v", "mkv", "mp4"];
const IMAGE_EXTS: [&str; 3] = ["jpeg", "jpg", "png"];
const AUDIO_EXTS: [&str; 3] = ["mp3", "ogg", "wav"];

#[derive(Debug)]
struct Component {
    total_size: u64,
    files: Vec<String>,
}

#[derive(Debug)]
struct Files {
    directories: Component,
    videos: Component,
    storyboards: Component,
    core_images: Component,
    core_sounds: Component,
    images: Component,
    sounds: Component,
}

pub struct Cleaner {
    args: Args,
}

impl Cleaner {
    pub fn init() -> Self {
        Self {
            args: Args::parse(),
        }
    }

    pub fn start(&self) -> Result<String> {
        let mut total_size: u64 = 0;

        for dir_entry in read_dir(&self.args.song_directory_path)? {
            let mut files = Files {
                directories: Component {
                    total_size: 0,
                    files: vec![],
                },
                storyboards: Component {
                    total_size: 0,
                    files: vec![],
                },
                videos: Component {
                    total_size: 0,
                    files: vec![],
                },
                core_images: Component {
                    total_size: 0,
                    files: vec![],
                },
                core_sounds: Component {
                    total_size: 0,
                    files: vec![],
                },
                images: Component {
                    total_size: 0,
                    files: vec![],
                },
                sounds: Component {
                    total_size: 0,
                    files: vec![],
                },
            };

            let beatmapset_path = dir_entry?.path();

            for entry in read_dir(&beatmapset_path)? {
                let file = entry?;

                let file_name = file
                    .file_name()
                    .to_str()
                    .context("Could not convert OsString to str!")?
                    .to_lowercase();

                // Add directories to file list as part of storyboards deletion feature
                if file.metadata()?.is_dir() {
                    files.directories.files.push(file_name);
                    files.directories.total_size += file.metadata()?.len();

                    continue;
                }

                // Parse file names from .osu files
                if file_name.ends_with(".osu") {
                    for mut line in read_to_string(file.path())?.split("\n") {
                        line = line.trim();

                        // Audio
                        if line.starts_with("AudioFilename:") {
                            let audio = line
                                .chars()
                                .skip(14)
                                .collect::<String>()
                                .trim()
                                .to_lowercase();

                            if !files.core_sounds.files.contains(&audio) {
                                files.core_sounds.total_size +=
                                    match File::open(beatmapset_path.join(&audio)) {
                                        Ok(audio_file) => audio_file.metadata()?.len(),
                                        Err(_) => 0,
                                    };

                                files.core_sounds.files.push(audio);
                            }
                        }

                        // Background
                        if line.starts_with("0,0,")
                            && IMAGE_EXTS.iter().any(|ext| line.contains(ext))
                        {
                            let mut background =
                                line.split(",").collect::<Vec<&str>>()[2].to_lowercase();

                            if background.starts_with('"') {
                                background = background
                                    .chars()
                                    .skip(1)
                                    .take(background.len() - 2)
                                    .collect();
                            }

                            if !files.core_images.files.contains(&background) {
                                // We'll ignore images that are inside another directory
                                if !background.contains(['/', '\\']) {
                                    files.core_images.total_size +=
                                        match File::open(beatmapset_path.join(&background)) {
                                            Ok(background_file) => {
                                                background_file.metadata()?.len()
                                            }
                                            Err(_) => 0,
                                        }
                                };

                                files.core_images.files.push(background);
                            }
                        }
                    }
                }
                // Add other files to file list
                else {
                    // Storyboards
                    if file_name.ends_with("osb") {
                        files.storyboards.total_size += file.metadata()?.len();
                        files.storyboards.files.push(file_name);
                    }
                    // Videos
                    else if VIDEO_EXTS.iter().any(|ext| file_name.ends_with(ext)) {
                        files.videos.total_size += file.metadata()?.len();
                        files.videos.files.push(file_name);
                    }
                    // Images
                    else if IMAGE_EXTS.iter().any(|ext| file_name.ends_with(ext)) {
                        files.images.total_size += file.metadata()?.len();
                        files.images.files.push(file_name);
                    }
                    // Sounds
                    else if AUDIO_EXTS.iter().any(|ext| file_name.ends_with(ext)) {
                        files.sounds.total_size += file.metadata()?.len();
                        files.sounds.files.push(file_name);
                    }
                }
            }

            // Filter out directories that contain core images from the directories component if exist
            if !self.args.backgrounds
                && files
                    .core_images
                    .files
                    .iter()
                    .any(|core_image| core_image.contains('/') || core_image.contains('\\'))
            {
                let directories = files
                    .core_images
                    .files
                    .iter()
                    .map(|core_image| {
                        core_image
                            .split(if core_image.contains('/') { '/' } else { '\\' })
                            .collect::<Vec<&str>>()[0]
                    })
                    .collect::<Vec<&str>>();

                files.directories.files = files
                    .directories
                    .files
                    .into_iter()
                    .filter(|directory| !directories.contains(&directory.as_str()))
                    .collect();
            }

            // Filter out core images from all images
            files.images.files = files
                .images
                .files
                .into_iter()
                .filter(|image| !files.core_images.files.contains(image))
                .collect();

            files.images.total_size -= files.core_images.total_size;

            // Filter out core sounds from all sounds
            files.sounds.files = files
                .sounds
                .files
                .into_iter()
                .filter(|sound| !files.core_sounds.files.contains(sound))
                .collect();

            files.sounds.total_size -= files.core_sounds.total_size;

            // Clean beatmapset
            total_size += self.clean(&beatmapset_path, &files)?;
        }

        Ok(format!(
            "Successfully saved ~{} MB!",
            total_size / 1024 / 1024
        ))
    }

    fn clean(&self, beatmapset_path: &PathBuf, files: &Files) -> Result<u64> {
        let mut total_size: u64 = 0;

        if self.args.debug {
            println!(
                "\n{}\n{:#?}\n",
                beatmapset_path
                    .to_str()
                    .context("Could not convert PathBuf to str!")?,
                files
            );
        }

        if self.args.backgrounds {
            total_size += self.bulk_remove(&beatmapset_path, &files.core_images, false)?;
        }

        if self.args.all || self.args.storyboards {
            total_size += self.bulk_remove(&beatmapset_path, &files.directories, true)?;
            total_size += self.bulk_remove(&beatmapset_path, &files.storyboards, false)?;
        }

        if self.args.all || self.args.videos {
            total_size += self.bulk_remove(&beatmapset_path, &files.videos, false)?;
        }

        if self.args.all || self.args.images {
            total_size += self.bulk_remove(&beatmapset_path, &files.images, false)?;
        }

        if self.args.all || self.args.sounds {
            total_size += self.bulk_remove(&beatmapset_path, &files.sounds, false)?;
        }

        Ok(total_size)
    }

    fn bulk_remove(
        &self,
        beatmapset_path: &PathBuf,
        component: &Component,
        is_dir: bool,
    ) -> Result<u64> {
        for file in &component.files {
            let path = beatmapset_path.join(file);
            let prettified_path = path.to_str().context("Could not convert PathBuf to str!")?;

            if !self.args.debug {
                match if is_dir {
                    remove_dir_all(&path)
                } else {
                    remove_file(&path)
                } {
                    Ok(_) => {
                        if !self.args.quiet {
                            println!("{}", prettified_path);
                        }
                    }
                    Err(err) => println!(
                        "An error occurred while trying to remove {}: {}",
                        prettified_path, err
                    ),
                }
            }
        }

        Ok(component.total_size)
    }
}
