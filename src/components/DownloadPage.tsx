import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Config } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { debug } from '@tauri-apps/plugin-log';
import { useState } from 'react';
import { Navigate, NavLink } from 'react-router';
import { Label } from "./ui/label";

const config: Config = await invoke('get_config');

const DownloadPage = ({ hasSeenHomepage }: { hasSeenHomepage: boolean }) => {
  const [url, setUrl] = useState('');
  const [quality, setQuality] = useState('Best');
  const [format, setFormat] = useState('MP4');
  const [isAdvancedOptionsOpen, setIsAdvancedOptionsOpen] = useState(false);
  const [advancedOptions, setAdvancedOptions] = useState({
    extractAudio: false,
    downloadSubtitles: false,
    downloadThumbnail: false
  });

    const handleDownload = async () => {
    try {
      const options = { 
        url, 
        quality, 
        format,
        ...advancedOptions
      };
      const result = await invoke('download_video', { options });
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
          className="flex-grow"
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

      <Dialog open={isAdvancedOptionsOpen} onOpenChange={setIsAdvancedOptionsOpen}>
        <DialogTrigger asChild>
          <Button variant="outline" className="w-full">Advanced Options</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Advanced Download Options</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="extract-audio"
                checked={advancedOptions.extractAudio}
                onCheckedChange={(checked) => setAdvancedOptions(prev => ({
                  ...prev,
                  extractAudio: !!checked
                }))}
              />
              <Label htmlFor="extract-audio">Extract Audio Only</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="subtitles"
                checked={advancedOptions.downloadSubtitles}
                onCheckedChange={(checked) => setAdvancedOptions(prev => ({
                  ...prev,
                  downloadSubtitles: !!checked
                }))}
              />
              <Label htmlFor="subtitles">Download Subtitles</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="thumbnail"
                checked={advancedOptions.downloadThumbnail}
                onCheckedChange={(checked) => setAdvancedOptions(prev => ({
                  ...prev,
                  downloadThumbnail: !!checked
                }))}
              />
              <Label htmlFor="thumbnail">Download Thumbnail</Label>
            </div>
            <NavLink to="/starter" className="menu-button ms-2 flex flex-col">
                Back to Main Menu
            </NavLink>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default DownloadPage;
