import React from "react";
import ReactDOM from "react-dom/client";
import { Overlay } from "./components/Overlay";
import "./styles/tokens.css";

document.documentElement.classList.add("overlay-page");
document.documentElement.style.background = "transparent";
document.body.style.background = "transparent";
document.body.style.margin = "0";
document.body.style.overflow = "hidden";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Overlay />
  </React.StrictMode>,
);
