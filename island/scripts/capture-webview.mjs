import fs from "fs";

const res = await fetch("http://127.0.0.1:9222/json");
const pages = await res.json();
const mainPage = pages.find(p => p.url === "http://localhost:1420/");

if (mainPage) {
    const ws = new WebSocket(mainPage.webSocketDebuggerUrl);
    await new Promise((resolve) => {
        ws.onopen = async () => {
            let msgId = 1;
            const send = (method, params = {}) => {
                const id = msgId++;
                return new Promise((r) => {
                    const handler = (event) => {
                        const data = JSON.parse(event.data);
                        if (data.id === id) {
                            ws.removeEventListener("message", handler);
                            r(data.result);
                        }
                    };
                    ws.addEventListener("message", handler);
                    ws.send(JSON.stringify({ id, method, params }));
                });
            };

            await send("Page.enable");
            const screenshot = await send("Page.captureScreenshot", { format: "png", fromSurface: true });
            if (screenshot && screenshot.data) {
                const buf = Buffer.from(screenshot.data, "base64");
                fs.writeFileSync("X:\\folder_ai\\Hime-Bloom\\webview-main-render.png", buf);
                console.log("Saved webview render to webview-main-render.png, bytes:", buf.length);
            } else {
                console.log("Capture screenshot returned empty:", screenshot);
            }
            ws.close();
            resolve();
        };
    });
}
