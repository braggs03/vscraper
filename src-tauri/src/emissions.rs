#[allow(dead_code)]
pub enum Emission {
    FfmpegInstall,
    YtdlpCancelDownload,
    YtdlpDownloadUpdate,
    YtdlpInstall,
    YtdlpUrlUpdate,
    YtdlpDownloadFinish,
}

impl Emission {
    pub fn as_string(&self) -> &'static str {
        match self {
            Emission::FfmpegInstall => "ffmpeg_install",
            Emission::YtdlpCancelDownload => "ytdlp_cancel_download",
            Emission::YtdlpDownloadUpdate => "ytdlp_download_update",
            Emission::YtdlpDownloadFinish => "ytdlp_download_finish",
            Emission::YtdlpInstall => "ytdlp_install",
            Emission::YtdlpUrlUpdate => "ytdlp_url_update",
        }
    }
}
