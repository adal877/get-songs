use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
// use std::io::Error;
use std::path::{PathBuf};
use std::process::Command;
use colored::*;

pub fn println_err(msg: &str) {
    eprintln!("{}", msg.bright_red());
}

pub fn println_success(msg: &str) {
    eprintln!("{}", msg.bright_green());
}

pub fn println_alert(msg: &str) {
    eprintln!("{}", msg.bright_yellow());
}

fn get_song_title(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("yt-dlp")
        .arg("--dump-single-json")
        .arg(url)
        .output()?;

    if !output.status.success() {
        return Err(
            format!("yt-dlp failed to get info for: {}", url).into()
        );
    }

    let json_output = String::from_utf8(output.stdout)?;
    let v: Value = serde_json::from_str(&json_output)?;
    let title = v["title"].as_str().ok_or("Title field not found in JSON output")?.to_string();

    Ok(title)
}

pub fn download_song_handler(url: &str, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let song_title = get_song_title(url)?;

    let status = Command::new("yt-dlp")
        .arg("-x") // Extracts audio only
        .arg("--audio-format")
        .arg("wav") // Converts to this format
        .arg("-o")
        .arg(output) // Defines the output template
        .arg(url) // The video url
        .status()?;

    if status.success() {
        println_success(&format!("Finished downloading: {}", url));
    } else {
        println_err(&format!("Error downloading: {}, Status code: {}", url, status.code().unwrap_or(-1)));
    }

    Ok(())
}

pub fn download_path_handler(path: &PathBuf, author_name: String, playlist_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Builds the destiny path ~/Music/<Author>/<Playlist>
    let mut music_dir =
        dirs::audio_dir().unwrap_or_else(|| path.to_path_buf()).to_path_buf();
    music_dir.push(&author_name);
    music_dir.push(&playlist_name);

    // Builds the dir if it didnt exists
    std::fs::create_dir_all(&music_dir)?;
    println_alert(&format!("Saving into: {}", music_dir.display()));

    Ok(())
}

#[derive(Debug)]
pub enum DownloadStatus {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    YtDlpError(String),
    Success,
}

#[derive(Debug)]
pub struct DownloadResult {
    album_name: String,
    song_name: String,
    status: DownloadStatus,
}

impl DownloadResult {
    fn new(album_name: String, song_name: String, status: DownloadStatus) -> Self {
        Self {
            album_name,
            song_name,
            status,
        }
    }
}

// Struct to define the desired informations
#[derive(Serialize, Deserialize, Debug)]
struct Album {
    author_name: String,
    playlist_name: String,
    genre: String,
    comment: String,
}

// Main struct to define the cli parameters
#[derive(Serialize, Deserialize, Debug)]
struct DownloadEntry {
    save_to: PathBuf,
    url: String,
    album: Album,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// This must download the yt videos as songs from a given json-like format
struct Args {
    #[arg(short, long, group = "input")]
    json: Option<String>,

    // Path to the given .json content
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let json_data: String = if let Some(json_str) = args.json {
        json_str
    } else if let Some(file_path) = args.file {
        fs::read_to_string(file_path)?
    } else {
        // hopefully this shouldnt run due to the 'group' configurantion in the 'clap'
        println_err("Error: you must give --json or --file");
        std::process::exit(1);
    };

    // Deserialize the json string into a rust struct
    let entries: Vec<DownloadEntry> = serde_json::from_str(&json_data)?;
    // Handles the failed entries
    let mut download_status_entries: Vec<DownloadResult> = Vec::new();

    // Iterate over each entry to start the download
    for entry in entries {
        println_alert(
            &format!(
                "Processing: {} -> {}. To: {:?}",
                entry.album.playlist_name, entry.url, entry.save_to.display()
            )
        );

        let music_dir = match download_path_handler(&entry.save_to, entry.album.author_name.clone(), entry.album.playlist_name.clone()) {
            Ok(_) => {
                let mut dir = entry.save_to.clone();
                dir.push(&entry.album.author_name);
                dir.push(&entry.album.playlist_name);
                dir
            },
            Err(e) => {
                println_err(&format!("Failed to create directory: {}. Error: {}", entry.save_to.display(), e));
                download_status_entries.push(
                    DownloadResult::new(
                        entry.album.playlist_name.clone(),
                        entry.url.clone(),
                        DownloadStatus::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create directory"))
                    )
                );
                continue;
            }
        };

        match download_song_handler(&entry.url, &music_dir) {
            Ok(_) => {
                println_success(&format!("Successfully downloaded from: {}", entry.url));
                download_status_entries.push(
                    DownloadResult::new(
                        entry.album.playlist_name.clone(),
                        entry.url.clone(),
                        DownloadStatus::Success
                    )
                );
            },
            Err(e) => {
                println_err(&format!("Failed to download from: {}. Error: {}", entry.url, e));
                // error_entries.push(entry); // Collect the failed entry
            }
        }
    }

    println_alert(
        format!(
            "{} entries processed.\nEntries summary: {:?}",
            download_status_entries.len(),
            download_status_entries.iter().map(|e| (&e.album_name, &e.song_name, &e.status))
            ).as_str()
        );

    Ok(())
}
