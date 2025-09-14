use serde_json::Value;
// use std::io::Error;
use std::path::{PathBuf};
use std::process::Command;

use crate::structs::{DownloadResult, Track, Album};
use crate::enums::DownloadStatusEnum as DownloadStatus;
use crate::utils::{println_err, println_success, println_alert};

/*
* Gets the playlist json from yt-dlp
* and deserializes it into a Value struct
* @param url: &str - The url to get the json from
* @return Result<Value, Box<dyn std::error::Error>> - The deserialized json or an error
*/
pub fn deserialize_ytdlp_handler(playlist_url: &str) -> Result<Value, Box<dyn std::error::Error>> {
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

pub fn download_path_handler(path: &PathBuf, author_name: String, album_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Builds the destiny path ~/Music/<Author>/<Playlist>
    let mut music_dir =
        dirs::audio_dir().unwrap_or_else(|| path.to_path_buf()).to_path_buf();
    music_dir.push(&author_name);
    music_dir.push(&album_name);

    // Builds the dir if it didnt exists
    std::fs::create_dir_all(&music_dir)?;
    println_alert(&format!("Saving into: {}", music_dir.display()));

    Ok(())
}

pub fn download_track_helper(track: Track, album: Album, album_dir: PathBuf) -> Result<DownloadResult, Box<dyn std::error::Error>> {
    let safe_track_name = track.track_name.replace("/", "_").replace("\\", "_");
    let mut output_path = album_dir.clone();
    output_path.push(format!("{}.%(ext)s", safe_track_name));

    match download_song_handler(&track.url, &output_path) {
        Ok(_) => {
            println_success(&format!("Successfully downloaded: {}", track.track_name));
            Ok(DownloadResult::new(
                album.album_name.clone(),
                track.track_name.clone(),
                album.author_name.clone(),
                album.genre.clone(),
                album.comment.clone(),
                DownloadStatus::Success
            ))
        },
        Err(e) => {
            println_err(&format!("Failed to download: {}. Error: {}", track.url, e));
            Ok(DownloadResult::new(
                album.album_name.clone(),
                track.track_name.clone(),
                album.author_name.clone(),
                album.genre.clone(),
                album.comment.clone(),
                DownloadStatus::YtDlpError(format!("Failed to download: {}", e))
            ))
        }
    }
}