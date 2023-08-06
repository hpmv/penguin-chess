import { App } from "./app";
import React from "react";
import ReactDOM from "react-dom/client";

window.worker = new Worker(new URL("./worker.js", import.meta.url));

const root = ReactDOM.createRoot(document.getElementById('app'));
root.render(React.createElement(App));
