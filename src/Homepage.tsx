import { Checkbox } from "@mui/material";
import "./App.css";
import { useState } from "react";
import { NavLink } from "react-router";
import { invoke } from '@tauri-apps/api/core';

const label = { inputProps: { 'aria-label': '' } };
const config: Config = await invoke('get_config');

const saveUserPreference = async (preference: boolean) => {

    const success = await invoke('set_homepage_preference', { preference });

    if (!success) {
        console.log("Failed to change user homepage preference.")
    }
}

export default function Homepage({ onGetStarted }: { onGetStarted: () => void }) {
    const [preference, setPreference] = useState(config.homepage_preference);

    return (
        <main className="flex flex-col items-center justify-center text-center min-h-screen">
            <h1 className="mb-5 font-sans text-3xl">Welcome to</h1>
            <div className="w-80">
                <img src="/vscraper-dark.svg" className="block dark:hidden w-full h-auto" alt="vscraper dark" />
                <img src="/vscraper-light.svg" className="hidden dark:block w-full h-auto" alt="vscraper light" />
            </div>
            <h1 className="m-5 font-sans text-3xl">A Simple Tool for Youtube DL's Weak Spots.</h1>
            <div className="flex flex-row">
                <a href="https://github.com/braggs03/vscraper" target="_blank" className="menu-button menu-button-padding ms-2">
                    Guide
                </a>
                <NavLink onClick={() => {
                    saveUserPreference(!preference);
                    onGetStarted();
                }} to="/" className="menu-button menu-button-padding ms-2">
                    Get Started
                </NavLink>
                <a href="https://github.com/braggs03/vscraper" target="_blank" className="menu-button menu-button-padding ms-2">
                    About
                </a>
            </div>
            <p className="text-xs pt-3">
                Don't show on start. <Checkbox
                    {...label}
                    size="small"
                    checked={preference}
                    onChange={(e) => setPreference(e.target.checked)}
                />
            </p>
        </main>
    );
}
