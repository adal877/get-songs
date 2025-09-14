
#[allow(dead_code)]
#[derive(Debug)]
pub enum DownloadStatusEnum {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    YtDlpError(String),
    Success,
    Pendent,
}
