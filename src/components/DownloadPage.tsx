import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Config } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { debug } from '@tauri-apps/plugin-log';
import { useState } from 'react';
import { Navigate, useNavigate } from 'react-router';
import { Label } from "./ui/label";

const config: Config = await invoke('get_config');

const DownloadPage = ({ hasSeenHomepage }: { hasSeenHomepage: boolean }) => {
    const [url, setUrl] = useState('');
    const navigate = useNavigate();
    const [quality, setQuality] = useState('Best');
    const [format, setFormat] = useState('MP4');
    const [isAdvancedOptionsOpen, setIsAdvancedOptionsOpen] = useState(false);
    const [advancedOptions, setAdvancedOptions] = useState({
        autoStart: 'Yes',
        downloadFolder: 'Default',
        customNamePrefix: 'Default',
        itemsLimit: 'Default',
        strictPlaylistMode: false
    });

    const handleDownload = async () => {
        try {
            const options = {
                url,
                quality,
                format,
                ...advancedOptions
            };
            const result = await invoke('download_best_quality', { options });
            console.log('Download started:', result);
        } catch (error) {
            console.error('Download failed:', error);
        }
    };

    if (!config.skip_homepage && !hasSeenHomepage) {
        debug("REDIRECT: /starter");
        return <Navigate to="/starter" />;
    }

    return (
        <div className="max-w-md mx-auto p-6 space-y-4">
            <div className="flex space-x-2">
                <Input
                    placeholder="Enter video or playlist URL"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    className="grow"
                />
                <Button onClick={handleDownload}>Download</Button>
            </div>

            <div className="flex space-x-2">
                <Select value={quality} onValueChange={setQuality}>
                    <SelectTrigger className="w-full">
                        <SelectValue placeholder="Quality" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="Best">Best</SelectItem>
                        <SelectItem value="1080p">1080p</SelectItem>
                        <SelectItem value="720p">720p</SelectItem>
                        <SelectItem value="480p">480p</SelectItem>
                    </SelectContent>
                </Select>

                <Select value={format} onValueChange={setFormat}>
                    <SelectTrigger className="w-full">
                        <SelectValue placeholder="Format" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="MP4">MP4</SelectItem>
                        <SelectItem value="MKV">MKV</SelectItem>
                        <SelectItem value="AVI">AVI</SelectItem>
                        <SelectItem value="WebM">WebM</SelectItem>
                    </SelectContent>
                </Select>
            </div>

            <Button 
                variant="outline" 
                className="w-full" 
                onClick={() => setIsAdvancedOptionsOpen(!isAdvancedOptionsOpen)}
            >
                Advanced Options
            </Button>

            {isAdvancedOptionsOpen && (
                <div className="space-y-4 p-4 border rounded-md">
                    <div className="flex space-x-2">
                        <div className="flex-1">
                            <Label>Auto Start</Label>
                            <Select 
                                value={advancedOptions.autoStart} 
                                onValueChange={(value) => setAdvancedOptions(prev => ({
                                    ...prev, 
                                    autoStart: value
                                }))}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="Auto Start" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="Yes">Yes</SelectItem>
                                    <SelectItem value="No">No</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="flex-1">
                            <Label>Download Folder</Label>
                            <Select 
                                value={advancedOptions.downloadFolder} 
                                onValueChange={(value) => setAdvancedOptions(prev => ({
                                    ...prev, 
                                    downloadFolder: value
                                }))}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="Download Folder" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="Default">Default</SelectItem>
                                    <SelectItem value="Custom">Custom</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <div className="flex space-x-2">
                        <div className="flex-1">
                            <Label>Custom Name Prefix</Label>
                            <Input 
                                placeholder="Default" 
                                value={advancedOptions.customNamePrefix}
                                onChange={(e) => setAdvancedOptions(prev => ({
                                    ...prev, 
                                    customNamePrefix: e.target.value
                                }))}
                            />
                        </div>
                        <div className="flex-1">
                            <Label>Items Limit</Label>
                            <Select 
                                value={advancedOptions.itemsLimit} 
                                onValueChange={(value) => setAdvancedOptions(prev => ({
                                    ...prev, 
                                    itemsLimit: value
                                }))}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="Items Limit" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="Default">Default</SelectItem>
                                    <SelectItem value="5">5</SelectItem>
                                    <SelectItem value="10">10</SelectItem>
                                    <SelectItem value="25">25</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <div className="flex items-center space-x-2">
                        <Checkbox
                            id="strict-playlist-mode"
                            checked={advancedOptions.strictPlaylistMode}
                            onCheckedChange={(checked) => setAdvancedOptions(prev => ({
                                ...prev,
                                strictPlaylistMode: !!checked
                            }))}
                        />
                        <Label htmlFor="strict-playlist-mode">Strict Playlist Mode</Label>
                    </div>

                    <div className="flex flex-col space-x-2 mt-4">
                        <Button variant="outline" className="w-full mb-2">Import URLs</Button>
                        <Button variant="outline" className="w-full mb-2">Export URLs</Button>
                        <Button variant="outline" className="w-full">Copy URLs</Button>
                    </div>
                </div>
            )}
            <Button variant="outline" className="w-full mb-2" onClick={() => navigate("/starter")}>
                Back to Main Menu
            </Button>
        </div>
    );
};

export default DownloadPage;
