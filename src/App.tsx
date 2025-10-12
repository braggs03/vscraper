import "./App.css";
import { Navigate } from "react-router";
import { invoke } from '@tauri-apps/api/core';
import { NavLink } from "react-router";
import { debug } from '@tauri-apps/plugin-log';

const config: Config = await invoke('get_config');

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
        </main>
    )
}