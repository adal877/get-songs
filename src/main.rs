mod handlers;
mod utils;
mod structs;
mod enums;

use handlers::*;
use utils::*;
use structs::*;

use clap::Parser;
use std::fs;


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

    println_alert(
        &format!(
            "json: {}",
            json_data
        )
    );

    // Deserialize the json string into a rust struct
    let download_entries: Vec<DownloadEntry> = serde_json::from_str(&json_data)?;
    // Handles the failed entries
    let mut download_status_entries = Vec::new();

    println_success(
        &format!(
            "json_data: {:?}",
            download_entries
        )
    );

    for entry in download_entries {
        println_alert(&format!("Processing playlist from: {}", entry.url));

        let playlist_json = deserialize_ytdlp_handler(&entry.url)?;
        println_success(&format!("Successfully fetched playlist info for: {}", entry.url));

        let album_entries = playlist_json["entries"].as_array().ok_or("Entries field not found or is not an array")?;
        let album_name = entry.album.playlist_name.unwrap_or_else(|| playlist_json["title"].as_str().unwrap_or("Unknown Album").to_string());
        let author_name = entry.album.author_name.clone();
        let genre = entry.album.genre.clone();
        let comment = entry.album.comment.clone().unwrap_or_else(|| "No comment provided".to_string());

        let tracks: Vec<Track> = album_entries.iter().filter_map(|item| {
            let video_url = item["url"].as_str()?.to_string();
            let track_name = item["title"].as_str()?.to_string();
            Some(Track {
                url: video_url,
                author_name: author_name.clone(),
                genre: genre.clone(),
                comment: Some(comment.clone()),
                track_name,
            })
        }).collect();

        let album = Album {
            url: entry.url.clone(),
            author_name: author_name.clone(),
            album_name: album_name.clone(),
            genre: genre.clone(),
            comment: comment.clone(),
            tracks,
        };

        let album_dir = match download_path_handler(
            &entry.save_to,
            album.author_name.clone(),
            album.album_name.clone()
        ) {
            Ok(_) => {
                let mut dir = entry.save_to.clone();
                dir.push(&album.author_name);
                dir.push(&album.album_name);
                dir
            },
            Err(e) => {
                println_err(&format!("Failed to create directory: {}. Error: {}", entry.save_to.display(), e));
                continue;
            }
        };

        // Iterate over the tracks in the playlist and download each one
        for track in &album.tracks {
            download_status_entries.push(
                download_track_helper(track.clone(), album.clone(), album_dir.clone())
            );
        }
    }

    println_alert(
        &format!(
            "{} entries processed.\nEntries summary: {:?}",
            download_status_entries.len(),
            download_status_entries
        )
    );

    Ok(())
}
