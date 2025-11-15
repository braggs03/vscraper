import "./App.css";
import { Navigate } from "react-router";
import { invoke } from '@tauri-apps/api/core';
import { NavLink } from "react-router";
import { debug } from '@tauri-apps/plugin-log';
import { listen } from "@tauri-apps/api/event";
import { ToastContainer, toast } from 'react-toastify';
import { useMantineColorScheme, useMantineTheme } from "@mantine/core";

const config: Config = await invoke('get_config');

const installDependencies = () => invoke("install_ffmpeg_ytdlp");

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

    const { colorScheme, setColorScheme } = useMantineColorScheme();

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

            <ToastContainer
                position="top-right" 
                autoClose={5000}
                theme={ colorScheme == "dark" ? "dark" : "light" }
            />
        </main>
    )
}