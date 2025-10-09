import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import { useState } from "react";
import Homepage from "./Homepage";
import App from "./App";

function Root() {
  const [hasSeenHomepage, setHasSeenHomepage] = useState(false);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App hasSeenHomepage={hasSeenHomepage} />} />
        <Route path="/starter" element={<Homepage onGetStarted={() => setHasSeenHomepage(true)} />} />
      </Routes>
    </BrowserRouter>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<Root />);
