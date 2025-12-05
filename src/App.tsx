import "./App.css";
import { Navigate } from "react-router";
import { invoke } from '@tauri-apps/api/core';
import { NavLink } from "react-router";
import { debug } from '@tauri-apps/plugin-log';
import { listen } from "@tauri-apps/api/event";
import { ToastContainer, toast } from 'react-toastify';
import { Button } from "@/components/ui/button"
import { useTheme } from "./components/theme-provider";
import { useState } from "react";

const config: Config = await invoke('get_config');

const installDependencies = () => invoke("install_ffmpeg_ytdlp");

const downloadVideo = async () => {
    await invoke("download_best_quality", { url: "https://www.youtube.com/watch?v=cl_s-RazjHw" }).then((message) => console.log(message));
}

listen<string>("ffmpeg_install", (success) => {
    if (success) {
        toast.success("Successfully Installed FFMPEG!");
    } else {
        toast.error("Failed to Install FFMPEG!");
    }
});

listen<string>("yt-dlp_install", (success) => {
    if (success) {
        toast.success("Successfully Installed YT-DLP!");
    } else {
        toast.error("Failed to Install YT-DLP!");
    }
});

export default function App({ hasSeenHomepage }: { hasSeenHomepage: boolean }) {

    const { theme } = useTheme();

    listen<DownloadProgress>("ytdlp_download_update", (payload) => {
        try {
            let download_update: DownloadProgress = payload.payload;
            setDownload(download_update.percent);
        } catch(bog) {
            console.log(bog);
        }
    });

    const [download, setDownload] = useState("0.0");

    if (!config.skip_homepage && !hasSeenHomepage) {
        debug("REDIRECT: /starter");
        return <Navigate to="/starter" />;
    }

    return (
        <main className="flex flex-col items-center justify-center text-center min-h-screen">
            <NavLink to="/starter" className="menu-button ms-2 flex flex-col">
                Back to Main Menu
            </NavLink>

            <button onClick={() => {
                installDependencies();
            }} className="menu-button m-2">
                Install YT-DLP and FFMPEG
            </button>

            <Button variant="outline" onClick={() => {
                downloadVideo();
            }} className="m-2">
                Test YT-DLP
            </Button>

            <h2>
                {download}
            </h2>

            <ToastContainer
                position="top-right"
                autoClose={5000}
                theme={theme == "dark" ? "dark" : "light"}
            />
        </main>
    )
}