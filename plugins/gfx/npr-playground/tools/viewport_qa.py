"""Basic Mode real-window smoke test (Windows/WebView2 only)."""
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import urllib.request

import psutil
from viewport_benchmark import Client, find_native


def launch(output, env, scene="gallery"):
    log = output.with_suffix(".log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        ["target/playground/deps/amigo_app.exe", "--hosted", "--mod", "npr-playground", "--scene", scene],
        env=env, stdout=log, stderr=log, creationflags=subprocess.CREATE_NO_WINDOW,
    )
    return process, psutil.Process(process.pid), log


def connect(port):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            pages = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list"))
            page = next(page for page in pages if "tauri.localhost" in page["url"])
            return Client(page["webSocketDebuggerUrl"])
        except (OSError, StopIteration):
            time.sleep(0.1)
    raise TimeoutError("companion did not start")


def close(process, owner, log):
    try:
        native = find_native(owner)
        ctypes.windll.user32.GetAncestor.restype = wintypes.HWND
        parent = ctypes.windll.user32.GetAncestor(native, 2)
        ctypes.windll.user32.PostMessageW(parent, 0x10, 0, 0)
        deadline = time.monotonic() + 5
        while process.poll() is None and time.monotonic() < deadline:
            time.sleep(0.05)
    except (RuntimeError, psutil.Error):
        pass
    exited = process.poll() is not None
    try:
        children = owner.children(recursive=True)
    except psutil.NoSuchProcess:
        children = []
    for child in children:
        try:
            child.kill()
        except psutil.Error:
            pass
    if process.poll() is None:
        process.kill()
    process.wait()
    log.close()
    return exited


def click_card(client, selector, label):
    clicked = client.js(
        f"(()=>{{const card=[...document.querySelectorAll('{selector}')].find(item=>item.textContent.toLowerCase().includes({json.dumps(label.lower())}));if(!card)return false;card.click();return true;}})()"
    )
    if not clicked:
        raise AssertionError(f"missing Basic Mode card: {label}")


def main():
    output = Path("target/viewport-qa")
    output.mkdir(exist_ok=True)
    probe = socket.socket()
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
    probe.close()
    env = os.environ.copy()
    env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = (
        f"--remote-debugging-port={port} --remote-debugging-address=127.0.0.1 "
        "--remote-allow-origins=http://localhost"
    )
    env["WEBVIEW2_USER_DATA_FOLDER"] = str((output / f"webview-{os.getpid()}").resolve())
    process, owner, log = launch(output / "basic", env)
    report = {}
    try:
        client = connect(port)
        report["page_url"] = client.js("location.href")
        report["initial_dom"] = client.js("document.body.innerText.slice(0, 500)")
        client.until("!!document.querySelector('.basic-workspace')", 30)
        client.until("document.querySelector('.viewport-hud')?.textContent.includes('Drawing ready')", 30)
        client.until("document.querySelectorAll('.model-card').length >= 2", 30)
        report["history_controls"] = client.js(
            "['Undo','Redo'].every(label=>[...document.querySelectorAll('.topbar button')].some(button=>button.textContent.trim()===label))"
        )
        report["basic_workspace_ready"] = True

        click_card(client, ".model-card", "sphere")
        client.until("document.querySelector('.document-name')?.textContent.toLowerCase().includes('sphere')", 15)
        report["model_selection"] = True

        click_card(client, ".look-card", "Pencil Study")
        client.until("[...document.querySelectorAll('.look-card')].some(card=>card.classList.contains('active')&&card.textContent.includes('Pencil Study'))", 15)
        report["look_selection"] = True

        report["viewport_status"] = client.js("document.querySelector('.viewport-hud')?.textContent")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        report["closing_tauri_stops_engine"] = close(process, owner, log)
        (output / "report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report))
    assert all(value for value in report.values() if isinstance(value, bool)), report


if __name__ == "__main__":
    main()
