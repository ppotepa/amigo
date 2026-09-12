"""Basic Mode real-window benchmark (Windows/WebView2 only).

This deliberately measures only controls that exist in the Basic Mode client:
opening one source model, applying one saved look, and receiving its viewport.
It never presses Save and uses a fresh process per source, so it cannot create
Drafts or rewrite authored scene profiles.
"""
import argparse
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
import websocket


class Client:
    def __init__(self, url):
        self.connection = websocket.create_connection(url, origin="http://localhost", timeout=30)
        self.sequence = 0

    def call(self, method, **params):
        self.sequence += 1
        self.connection.send(json.dumps({"id": self.sequence, "method": method, "params": params}))
        while True:
            result = json.loads(self.connection.recv())
            if result.get("id") == self.sequence:
                if "error" in result:
                    raise RuntimeError(result["error"])
                return result.get("result", {})

    def js(self, expression):
        result = self.call("Runtime.evaluate", expression=expression, returnByValue=True, awaitPromise=True)
        if "exceptionDetails" in result:
            raise RuntimeError(result["exceptionDetails"])
        return result.get("result", {}).get("value")

    def until(self, expression, timeout=15):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.js(expression):
                return
            time.sleep(0.05)
        raise TimeoutError(expression)

    def close(self):
        self.connection.close()


def find_native(process):
    handles = []
    pids = {process.pid, *(child.pid for child in process.children(recursive=True))}
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    def visit(hwnd, _):
        pid = wintypes.DWORD()
        ctypes.windll.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value in pids:
            def child(child_hwnd, _):
                name = ctypes.create_unicode_buffer(256)
                ctypes.windll.user32.GetClassNameW(child_hwnd, name, 256)
                if name.value == "AmigoPlaygroundViewport":
                    handles.append(child_hwnd)
                return True
            ctypes.windll.user32.EnumChildWindows(hwnd, callback_type(child), 0)
        return True

    ctypes.windll.user32.EnumWindows(callback_type(visit), 0)
    if not handles:
        raise RuntimeError("Native child HWND was not created")
    return handles[0]


def resources(process):
    result = []
    for item in [process] + process.children(recursive=True):
        try:
            memory = item.memory_info()
            result.append({
                "pid": item.pid,
                "name": item.name(),
                "handles": item.num_handles(),
                "rss": memory.rss,
                "private": memory.private,
            })
        except psutil.Error:
            pass
    return result


def launch(executable, output, env):
    log = output.with_suffix(".log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        [executable, "--hosted", "--mod", "npr-playground", "--scene", "gallery"],
        env=env,
        stdout=log,
        stderr=log,
        creationflags=subprocess.CREATE_NO_WINDOW,
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


def click_card(client, selector, label):
    label = label.lower().replace("-", " ")
    clicked = client.js(
        "(()=>{const card=[...document.querySelectorAll(" + json.dumps(selector) + ")]"
        ".find(item=>item.textContent.toLowerCase().includes(" + json.dumps(label) + "));"
        "if(!card)return false;card.click();return true;})()"
    )
    if not clicked:
        raise AssertionError(f"missing Basic Mode card: {label}")


def close(process, owner, log):
    try:
        native = find_native(owner)
        ctypes.windll.user32.GetAncestor.restype = wintypes.HWND
        ctypes.windll.user32.PostMessageW(ctypes.windll.user32.GetAncestor(native, 2), 0x10, 0, 0)
        process.wait(timeout=5)
    except (RuntimeError, psutil.Error, subprocess.TimeoutExpired):
        pass
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


def run_case(executable, output, model, look, seconds):
    probe = socket.socket()
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
    probe.close()
    env = os.environ.copy()
    env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = (
        f"--remote-debugging-port={port} --remote-debugging-address=127.0.0.1 "
        "--remote-allow-origins=http://localhost"
    )
    env["WEBVIEW2_USER_DATA_FOLDER"] = str((output / f"webview-{model}-{look}-{os.getpid()}").resolve())
    started = time.monotonic()
    process, owner, log = launch(executable, output / f"{model}-{look}", env)
    client = None
    try:
        client = connect(port)
        client.until("!!document.querySelector('.basic-workspace')", 30)
        client.until("document.querySelector('.viewport-hud')?.textContent.includes('Drawing ready')", 30)
        client.until("document.querySelectorAll('.model-card').length >= 2", 30)
        click_card(client, ".model-card", model)
        client.until(
            "document.querySelector('.document-name')?.textContent.toLowerCase().includes(" + json.dumps(model.lower()) + ")",
            15,
        )
        click_card(client, ".look-card", look)
        client.until(
            "[...document.querySelectorAll('.look-card')].some(card=>card.classList.contains('active')"
            "&&card.textContent.toLowerCase().includes(" + json.dumps(look.lower().replace("-", " ")) + "))",
            15,
        )
        client.until("document.querySelector('.viewport-hud')?.textContent.includes('Drawing ready')", 30)
        time.sleep(seconds)
        return {
            "kind": "basic-mode-case",
            "model": model,
            "look": look,
            "ready_seconds": time.monotonic() - started,
            "viewport_status": client.js("document.querySelector('.viewport-hud')?.textContent"),
            "document_status": client.js("document.querySelector('.document-status')?.textContent"),
            "resources": resources(owner),
            "valid": True,
        }
    finally:
        if client:
            client.close()
        close(process, owner, log)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", default="target/playground/deps/amigo_app.exe")
    parser.add_argument("--seconds", type=float, default=2)
    parser.add_argument("--models", default="cube,sphere,suzanne")
    parser.add_argument("--looks", default="comic-ink,pencil-study,watercolour-wash")
    parser.add_argument("--output", default="target/viewport-basic-mode.jsonl")
    args = parser.parse_args()
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    for model in args.models.split(","):
        for look in args.looks.split(","):
            try:
                record = run_case(args.executable, output.parent, model, look, args.seconds)
            except Exception as error:
                record = {"kind": "error", "model": model, "look": look,
                          "error_type": type(error).__name__, "error": str(error), "valid": False}
            with output.open("a", encoding="utf-8") as results:
                results.write(json.dumps(record) + "\n")
            print(json.dumps(record), flush=True)
            if not record["valid"]:
                raise SystemExit(2)


if __name__ == "__main__":
    main()
