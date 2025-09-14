use crate::enums::DownloadStatusEnum as DownloadStatus;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResult {
    pub album_url: String,
    pub album_name: String,
    pub song_name: String,
    pub song_url: String,
    pub author_name: String,
    pub genre: String,
    pub comment: String,
    pub status: DownloadStatus,
}

impl DownloadResult {
    pub fn new(album_url: String, album_name: String, song_name: String, song_url: String, author_name: String, genre: String, comment: String, status: DownloadStatus) -> Self {
        Self {
            album_url,
            album_name,
            song_name,
            song_url,
            author_name,
            genre,
            comment,
            status,
        }
    }
}

// Struct to define the desired informations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Track {
    pub url: String,
    pub author_name: String,
    pub track_name: String,
    pub genre: String,
    pub comment: Option<String>,
}

#[allow(dead_code)]
impl Track {
    pub fn new(url: String, author_name: String, track_name: String, genre: String, comment: String) -> Self {
        Self { url, author_name, track_name, genre, comment: Some(comment) }
    }
}

// Struct to define the desired informations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Album {
    pub url: String,
    pub author_name: String,
    pub album_name: String,
    pub genre: String,
    pub comment: String,
    pub tracks: Vec<Track>,
}

#[allow(dead_code)]
impl Album {
    pub fn new(url: String, author_name: String, album_name: String, genre: String, comment: String, tracks: Vec<Track>) -> Self {
        Self { url, author_name, album_name, genre, comment, tracks}
    }
}

// Struct para representar o payload de um álbum
// Corresponde à estrutura do JSON de entrada
#[derive(Serialize, Deserialize, Debug)]
pub struct AlbumPayload {
    pub author_name: String,
    pub playlist_name: Option<String>,
    pub genre: String,
    pub comment: Option<String>,
}

// Struct principal para deserializar o JSON de entrada
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadEntry {
    pub save_to: PathBuf,
    pub url: String,
    pub album: AlbumPayload,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// This must download the yt videos as songs from a given json-like format
pub struct Args {
    #[arg(short, long, group = "input")]
    pub json: Option<String>,

    // Path to the given .json content
    #[arg(short, long, group = "input")]
    pub file: Option<PathBuf>,
}

