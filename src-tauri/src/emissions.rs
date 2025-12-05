pub enum Emissions {
    YtdlpUrlSuccess,
    YtdlpDownloadUpdate,
}

impl Emissions {
    pub fn as_str(&self) -> &'static str {
        match self {
            Emissions::YtdlpUrlSuccess => "ytdlp_url_success",
            Emissions::YtdlpDownloadUpdate => "ytdlp_download_update",
        }
    }
}
