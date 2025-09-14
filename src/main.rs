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


#[derive(Debug)]
pub enum DownloadStatus {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    YtDlpError(String),
    Success,
    Pendent,
}

#[derive(Debug)]
pub struct DownloadResult {
    album_name: String,
    song_name: String,
    author_name: String,
    genre: String,
    comment: String,
    status: DownloadStatus,
}

impl DownloadResult {
    pub fn new(album_name: String, song_name: String, status: DownloadStatus) -> Self {
        Self {
            album_name,
            song_name,
            author_name: String::new(),
            genre: String::new(),
            comment: String::new(),
            status,
        }
    }
}

// Struct to define the desired informations
#[derive(Serialize, Deserialize, Debug)]
struct Track {
    url: String,
    author_name: String,
    track_name: String,
    genre: String,
    comment: Option<String>,
}

impl Track {
    pub fn new(url: String, author_name: String, track_name: String, genre: String, comment: String) -> Self {
        Self { url, author_name, track_name, genre, comment: Some(comment) }
    }
}

// Struct to define the desired informations
#[derive(Serialize, Deserialize, Debug)]
struct Album {
    url: String,
    author_name: Option<String>,
    album_name: String,
    genre: String,
    comment: Option<String>,
    tracks: Vec<Track>,
}

impl Album {
    pub fn new(url: String, author_name: String, album_name: String, genre: String, comment: String, tracks: Vec<Track>) -> Self {
        Self { url, author_name: Some(author_name), album_name, genre, comment: Some(comment), tracks}
    }
}

// Main struct to define the cli parameters
#[derive(Serialize, Deserialize, Debug)]
struct DownloadEntry {
    save_to: PathBuf,
    url: String,
    album: Album,
}

impl DownloadEntry {
    pub fn new(save_to: PathBuf, url: String, album: Album) -> Self {
        Self { save_to, url, album }
    }
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

/*
* Gets the playlist json from yt-dlp
* and deserializes it into a Value struct
* @param url: &str - The url to get the json from
* @return Result<Value, Box<dyn std::error::Error>> - The deserialized json or an error
*/
fn deserialize_ytdlp_handler(playlist_url: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new("yt-dlp")
        .arg("--flat-playlist")
        .arg("--dump-single-json")
        .arg(playlist_url)
        .output()?;

    if !output.status.success() {
        return Err(
            format!("yt-dlp failed to get info for: {}", playlist_url).into()
        );
    }

    let json_output = String::from_utf8(output.stdout)?;
    let v: Value = serde_json::from_str(&json_output)?;
    // let title = v["title"].as_str().ok_or("Title field not found in JSON output")?.to_string();

    Ok(v)
}

pub fn download_song_handler(url: &str, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {

    let status = Command::new("yt-dlp")
        .arg("--ignore-errors")
        .arg("--format")
        .arg("bestaudio")
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("wav")
        .arg("--audio-quality")
        .arg("160k")
        .arg("--output")
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
    let album_to_download: Vec<DownloadEntry> = serde_json::from_str(&json_data)?;
    // Handles the failed entries
    let mut download_status_entries: Vec<DownloadResult> = Vec::new();

    // Iterate over each entry/album to start the download
    for entry in album_to_download {
        println_alert(
            &format!(
                "Processing: {} -> {}. To: {:?}",
                entry.album.album_name, entry.url, entry.save_to.display()
            )
        );

        // Iterate over each song in the album
        match deserialize_ytdlp_handler(&entry.url) {
            Ok(playlist_json) => {
                println_success(&format!("Successfully fetched playlist info for: {}", entry.url));

                let album_entries: &Vec<Value> = playlist_json["entries"].as_array().ok_or("Entries field not found or is not an array")?;
                let author_name = entry.album.author_name.clone().unwrap_or_else(|| album_entries[0]["uploader"].as_str().unwrap_or("Unknown Artist").to_string());
                let album_name = entry.album.album_name.clone();

                let tracks: Vec<Track> = album_entries.iter().filter_map(|item| {
                    let video_url = item["url"].as_str()?.to_string();
                    let track_name = item["title"].as_str()?.to_string();
                    Some(Track::new(
                        video_url.clone(),
                        author_name.clone(),
                        track_name,
                        entry.album.genre.clone(),
                        video_url,
                    ))
                }).collect();
                let album = Album::new(
                    entry.url.clone(),
                    author_name.clone(),
                    album_name.clone(),
                    entry.album.genre.clone(),
                    entry.url.clone(),
                    tracks,
                );

                let album_dir = match download_path_handler(
                    &entry.save_to,
                    album.author_name.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
                    album.album_name.clone()
                ) {
                    Ok(_) => {
                        let mut dir = entry.save_to.clone();
                        dir.push(&album.author_name.clone().unwrap_or_else(|| "Unknown Artist".to_string()));
                        dir.push(&album.album_name.clone());
                        dir
                    },
                    Err(e) => {
                        println_err(&format!("Failed to create directory: {}. Error: {}", entry.save_to.display(), e));
                        continue;
                    }
                };

                // Iterate over each track and download it
                for track in &album.tracks {
                    let mut output_path = album_dir.clone();
                    let safe_track_name = track.track_name.replace("/", "_").replace("\\", "_");
                    output_path.push(format!("{}.%(ext)s", safe_track_name));

                    match download_song_handler(&track.url, &output_path) {
                        Ok(_) => {
                            println_success(&format!("Successfully downloaded: {}", track.track_name));
                            download_status_entries.push(
                                DownloadResult::new(
                                    album.album_name.clone(),
                                    track.track_name.clone(),
                                    DownloadStatus::Success
                                )
                            );
                        },
                        Err(e) => {
                            println_err(&format!("Failed to download: {}. Error: {}", track.url, e));
                            download_status_entries.push(
                                DownloadResult::new(
                                    album.album_name.clone(),
                                    track.track_name.clone(),
                                    DownloadStatus::YtDlpError(format!("Failed to download: {}", e))
                                )
                            );
                        }
                    }
                } // End of track iteration
            },
            Err(e) => {
                println_err(&format!("Failed to fetch playlist info for: {}. Error: {}", entry.url, e));
                continue;
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
