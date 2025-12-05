interface Config {
    skip_homepage: boolean,
}

interface DownloadProgress {
    url: string,
    percent: string,
    size_downloaded: string,
    speed: string,
    eta: string,
}