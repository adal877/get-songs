use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub enum DownloadStatusEnum {
    IoError(String),        // Store error description as String
    JsonError(String),      // Store error description as String
    YtDlpError(String),
    Success,
    Pendent,
}
