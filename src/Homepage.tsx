import "./App.css";
import { useNavigate } from "react-router";
import { Checkbox } from "@mui/material";
import { useEffect, useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { useTheme } from "./components/theme-provider";
import { Button } from "./components/ui/button";
import { Config } from "./types";

const getPreference = async () => {
    const config: Config = await invoke('get_config', {});
    let preference = config.skip_homepage;
    return preference;
}

const updatePreference = async (updatedPreference: boolean) => {
    const config: Config = await invoke('get_config', {});
    config.skip_homepage = updatedPreference;
    let updatedConfig = config;
    const success = await invoke('update_config', { updatedConfig });

    return success;
}

export default function Homepage({ onGetStarted }: { onGetStarted: () => void }) {

    let navigate = useNavigate();

    const { theme } = useTheme();

    const [preference, setPreference] = useState(false);

    useEffect(() => {
        getPreference().then(setPreference);
    }, []);

    return (
        <main className="flex flex-col items-center justify-center text-center min-h-screen">
            <h1 className="mb-5 font-sans text-3xl">Welcome to</h1>
            <div className="w-80">
                {
                    theme ?
                        <img src={"/vscraper-dark.svg"} className="block dark:hidden w-full h-auto" alt="vscraper dark" />
                        :
                        <img src={"/vscraper-light.svg"} className="block dark:hidden w-full h-auto" alt="vscraper dark" />
                }
            </div>
            <h1 className="m-5 font-sans text-3xl">A Simple Tool for Youtube DL's Weak Spots.</h1>
            <div className="flex flex-row">
                <Button className="mr-1">
                    <a href="https://github.com/braggs03/vscraper" target="_blank">
                        About
                    </a>
                </Button>
                <Button variant="outline" onClick={() => {
                    onGetStarted();
                    navigate("/");
                }} className="mr-1">
                    Start
                </Button>
                <Button variant="outline">
                    <a href="https://github.com/braggs03/vscraper" target="_blank" className="">
                        Guide
                    </a>
                </Button>
            </div>
            <p className="text-sm pt-3">
                Don't show on start. <Checkbox
                    size="small"
                    checked={preference}
                    onChange={(e) => {
                        setPreference(e.target.checked);
                        updatePreference(e.target.checked);
                    }}
                />
            </p>
        </main>
    );
}
