pub enum Emission {
    FfmpegInstall,
    YtdlpDownloadUpdate,
    YtdlpInstall,
    YtdlpUrlSuccess,
}

impl Emission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Emission::FfmpegInstall => "ffmpeg_install",
            Emission::YtdlpDownloadUpdate => "ytdlp_download_update",
            Emission::YtdlpInstall => "ytdlp_install",
            Emission::YtdlpUrlSuccess => "ytdlp_url_success",
        }
    }
}
