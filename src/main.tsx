import ReactDOM from "react-dom/client";
import '@mantine/core/styles.css';
import { BrowserRouter, Route, Routes } from "react-router";
import { useState } from "react";
import Homepage from "./Homepage";
import App from "./App";
import { MantineProvider } from '@mantine/core';



function Root() {
    const [hasSeenHomepage, setHasSeenHomepage] = useState(false);

    return (
        <MantineProvider>
            <BrowserRouter>
                <Routes>
                    <Route path="/" element={<App hasSeenHomepage={hasSeenHomepage} />} />
                    <Route path="/starter" element={<Homepage onGetStarted={() => setHasSeenHomepage(true)} />} />
                </Routes>
            </BrowserRouter>
        </MantineProvider>
    );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<Root />);
