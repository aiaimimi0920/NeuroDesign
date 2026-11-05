import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import { Button, NeuroProvider, ToastProvider, type NeuroMode } from "@neuro/ui";
import "@neuro/ui/styles.css";
import "./preview.css";
import { Gallery } from "./Gallery";
import { Website } from "./Website";

function Preview() {
  const [mode, setMode] = useState<NeuroMode>(new URLSearchParams(location.search).get("mode") === "light" ? "light" : "dark");
  const website = document.body.dataset.page === "website";
  return <NeuroProvider mode={mode}><ToastProvider>
    <a className="preview-skip" href="#main-content">跳到内容</a>
    <header className="preview-bar"><nav aria-label="预览导航">
      <a className="preview-brand" href="../../neuro.html">neuro</a>
      <a href="index.html" aria-current={!website ? "page" : undefined}>控件</a>
      <a href="example.html" aria-current={website ? "page" : undefined}>网站示例</a>
      <a href="DESIGN.zh-CN.md">规范</a>
      <a href="../../packages/react/neuro-ui/README.md">React 接入</a>
    </nav><Button compact variant="quiet" onClick={() => setMode(mode === "dark" ? "light" : "dark")}>{mode === "dark" ? "浅色" : "深色"}</Button></header>
    {website ? <Website /> : <Gallery />}
  </ToastProvider></NeuroProvider>;
}

const root = document.getElementById("app");
if (!root) throw new Error("Preview root element is missing");
createRoot(root).render(<StrictMode><Preview /></StrictMode>);
