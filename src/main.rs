mod handlers;
mod utils;
mod structs;
mod enums;

use handlers::*;
use rusqlite::Connection;
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

    // Deserialize the json string into a rust struct
    let download_entries: Vec<DownloadEntry> = serde_json::from_str(&json_data)?;
    // Handles the failed entries
    let mut download_status_entries = Vec::new();

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

    let sqlitedb_connection = Connection::open("songs.db")?;

    sqlitedb_connection.execute(
        "CREATE TABLE IF NOT EXISTS download_status (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  album_url TEXT NOT NULL,
                  album_name TEXT NOT NULL,
                  song_name TEXT NOT NULL,
                  song_url TEXT NOT NULL,
                  status TEXT NOT NULL,
                  author_name TEXT NOT NULL,
                  genre TEXT NOT NULL,
                  comment TEXT NOT NULL
                  )",
        [],
    )?;
    for result in &download_status_entries {
        if let Ok(download_result) = result {
            sqlitedb_connection.execute(
                "INSERT INTO download_status (album_url, album_name, song_name, song_url, status, author_name, genre, comment) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [
                    &download_result.album_url,
                    &download_result.album_name,
                    &download_result.song_name,
                    &download_result.song_url,
                    &format!("{:?}", download_result.status),
                    &download_result.author_name,
                    &download_result.genre,
                    &download_result.comment,
                ],
            )?;
        }
        println_alert("##################################################");
        println!("Inserted download result into database: {:?}", result);
        println_alert("##################################################");
    }

    println_alert("##################################################");
    println_alert(
        &format!(
            "{} entries processed.\nEntries summary: {:?}",
            download_status_entries.len(),
            download_status_entries
        )
    );
    println_alert("##################################################");

    Ok(())
}
