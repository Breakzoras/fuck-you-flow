// Entry for the interface-picture build: the same App as src/main.tsx, drawn
// in a browser on the sample content of data.ts.
import React from "react";
import ReactDOM from "react-dom/client";
import App from "../../src/App";
import "../../src/App.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
