import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initWebVitals } from "./monitoring/webVitals";
import "./styles/tokens.css";
import "./styles.css";

initWebVitals();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
