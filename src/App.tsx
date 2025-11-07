import "./App.css";
import { Navigate } from "react-router";
import { invoke } from '@tauri-apps/api/core';
import { NavLink } from "react-router";
import { debug } from '@tauri-apps/plugin-log';
import { listen } from "@tauri-apps/api/event";
import { ToastContainer, toast } from 'react-toastify';

const config: Config = await invoke('get_config');

const installDependencies = () => invoke("install_yt_dlp_ffmpeg");

const submit_link = () => {
    
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

            <form action={submit_link}>
                <input type="text" />
                <input type="submit" value="Submit" />
            </form>

            <ToastContainer position="top-right" autoClose={5000}/>
        </main>
    )
}