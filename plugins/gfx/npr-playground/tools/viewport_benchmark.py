"""Exercise the real Windows Tauri companion through its local DevTools endpoint.

Requires Python websocket-client and psutil. Run from the repository root after
building amigo-app with --profile playground. Never saves the authored scene.
The debugging endpoint and isolated WebView profile exist only for this run.
"""
import argparse
import ctypes
from ctypes import wintypes
import json
import math
import os
from pathlib import Path
import socket
import statistics
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
        raise TimeoutError(expression + ": " + str(self.js("document.querySelector('footer')?.textContent")))

    def settled(self):
        self.until("!document.querySelector('.spin-toggle input').disabled")

    def tab(self, name):
        self.js("[...document.querySelectorAll('nav button')].find(x=>x.textContent===" + json.dumps(name) + ").click()")
        time.sleep(0.06)

    def checkbox(self, label, checked):
        self.settled()
        self.js("(()=>{const i=[...document.querySelectorAll('label')].find(x=>x.textContent.trim()===" + json.dumps(label) + ")?.querySelector('input');if(!i)throw Error('missing checkbox');if(i.checked!==" + json.dumps(checked) + ")i.click();})()")
        time.sleep(0.08)
        self.settled()

    def mode(self, mode):
        self.js("(()=>{const s=document.querySelector('.presentation select');s.value=" + json.dumps(mode) + ";s.dispatchEvent(new Event('change',{bubbles:true}));})()")
        name = {"native_gpu": "Native GPU", "local_rgba": "Local RGBA", "jpeg": "JPEG"}[mode]
        self.until("document.querySelector('.hud').textContent.startsWith(" + json.dumps(name) + ")")
        error = self.js("document.querySelector('.presentation-error')?.textContent")
        if error:
            raise RuntimeError(error)

    def model(self, model):
        self.tab("Model")
        while self.js("!!document.querySelector('button.danger')"):
            self.settled()
            self.js("document.querySelector('button.danger').click()")
            time.sleep(0.15)
        for name in (["cube", "sphere", "suzanne"] if model == "multi" else [model]):
            self.js("[...document.querySelectorAll('.model-card')].find(x=>x.querySelector('b').textContent===" + json.dumps(name) + ").click()")
            time.sleep(0.15)
            self.checkbox("Enable spinning animation", False)

    def look(self, look):
        self.tab("Look")
        self.settled()
        self.js("(()=>{const s=[...document.querySelectorAll('label')].find(x=>x.textContent.startsWith('Base look')).querySelector('select');s.value=" + json.dumps(look) + ";s.dispatchEvent(new Event('change',{bubbles:true}));})()")
        time.sleep(0.15)
        self.settled()

    def layers(self, count):
        self.settled()
        self.js("(()=>{const f=window.__setBenchmarkLayerCount;if(!f)throw Error('benchmark layer control unavailable');f(" + str(count) + ");})()")
        time.sleep(0.15)
        self.settled()


MEASURE = """(()=>{
  window.__measure={enabled:true,frames:[],inputs:{},errors:[]}; window.__viewportDiagnostics={};
  window.__pointerEvents=[];
  for(const type of ['pointerdown','pointermove','pointerup','pointercancel','lostpointercapture','blur'])window.addEventListener(type,e=>{if(__measure.enabled)__pointerEvents.push({type,target:e.target.className,focus:document.activeElement?.className});},true);
  window.addEventListener('playground-input-sent',event=>{if(__measure.enabled&&event.detail.kind==='navigate')__measure.inputs[event.detail.request_id]=event.detail.queued_at;});
  window.addEventListener('playground-frame-presented',event=>{
    if(!__measure.enabled)return;
    const header=event.detail,now=performance.now(),latencies=[];
    for(const [id,start] of Object.entries(__measure.inputs))if(+id<=header.input_id){latencies.push(now-start);delete __measure.inputs[id];}
    __measure.frames.push({time:now,latencies,header});
  });
  const send=WebSocket.prototype.send;
  WebSocket.prototype.send=function(data){
    if(!this.__measured){this.__measured=true;this.addEventListener('message',event=>{if(typeof event.data==='string'){
      const m=JSON.parse(event.data);if(m.type==='rejected'){delete __measure.inputs[m.request_id];__measure.errors.push(m.error);}
    }});}
    return send.call(this,data);
  };
})()"""


def percentile(values, quantile):
    return sorted(values)[min(len(values) - 1, math.ceil(len(values) * quantile) - 1)] if values else None


def summary(frames):
    intervals = [right["time"] - left["time"] for left, right in zip(frames, frames[1:])]
    latencies = [value for frame in frames for value in frame["latencies"]]
    return {"frames": len(frames), "fps": 1000 / statistics.mean(intervals) if intervals else 0,
            "interval_p95_ms": percentile(intervals, .95), "input_p95_ms": percentile(latencies, .95),
            "stages_mean_ms": {name: statistics.mean(frame["header"]["stages"][name] for frame in frames)
                               for name in frames[0]["header"]["stages"]} if frames else {}}


def measure(client, scenario, seconds, native_hwnd):
    client.checkbox("Enable spinning animation", scenario == "spin")
    client.tab("Motion")
    client.checkbox("Pause sketch", scenario == "pause")
    # Keep short matrix smoke runs fast while preserving the historical
    # settling window for the normal 1–3 second measurements.
    settle_seconds = max(0.15, min(1.2, seconds + 0.2))
    time.sleep(settle_seconds)
    client.js("__measure.frames=[];__measure.inputs={};__measure.errors=[];__pointerEvents=[];__viewportDiagnostics={}")
    start = time.monotonic()
    if scenario == "orbit":
        client.js("[...document.querySelectorAll('.camera-mode-row button')].find(x=>x.textContent==='Orbit').click()")
        rect = client.js("document.querySelector('.viewport').getBoundingClientRect().toJSON()")
        x, y = rect["x"] + 60, rect["y"] + 60
        native = client.js("document.querySelector('.presentation select').value==='native_gpu'")
        dpr = client.js("devicePixelRatio")
        def move(kind, px, py):
            if native:
                message = {"mousePressed": 0x201, "mouseMoved": 0x200, "mouseReleased": 0x202}[kind]
                packed = (round((py-rect['y'])*dpr)&0xffff)<<16 | (round((px-rect['x'])*dpr)&0xffff)
                ctypes.windll.user32.PostMessageW(native_hwnd, message, 0 if kind=='mouseReleased' else 1, packed)
            else:
                client.call("Input.dispatchMouseEvent", type=kind, x=px, y=py, button="left", buttons=0 if kind=='mouseReleased' else 1, clickCount=1 if kind!='mouseMoved' else 0)
        move('mousePressed', x, y)
        frame = 0
        while time.monotonic() - start < seconds:
            frame += 1
            move('mouseMoved',x + (frame % 180),y + 10 * math.sin(frame / 30))
            time.sleep(max(0, start + frame / 60 - time.monotonic()))
        move('mouseReleased',x + (frame % 180),y)
    else:
        time.sleep(seconds)
    result = client.js("({frames:__measure.frames,errors:__measure.errors,diagnostics:__viewportDiagnostics})")
    measurement=summary(result['frames'])
    pointer_events = client.js('__pointerEvents') if scenario == 'orbit' else []
    short_orbit_smoke = scenario == 'orbit' and seconds < 0.5 and len(pointer_events) >= 2
    return {**measurement, "valid":scenario!='orbit' or measurement['input_p95_ms'] is not None or short_orbit_smoke, "rejections": result["errors"], "raw": result["frames"], "pointer_events": pointer_events, "diagnostics": result["diagnostics"]}


def find_native(process):
    handles=[]
    pids={process.pid, *(child.pid for child in process.children(recursive=True))}
    callback_type=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
    def visit(hwnd, _):
        pid=wintypes.DWORD();ctypes.windll.user32.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
        if pid.value in pids:
            def child(hwnd, _):
                name=ctypes.create_unicode_buffer(256);ctypes.windll.user32.GetClassNameW(hwnd,name,256)
                if name.value=='AmigoPlaygroundViewport':handles.append(hwnd)
                return True
            ctypes.windll.user32.EnumChildWindows(hwnd,callback_type(child),0)
        return True
    ctypes.windll.user32.EnumWindows(callback_type(visit),0)
    if not handles:raise RuntimeError('Native child HWND was not created')
    ctypes.windll.user32.PostMessageW.argtypes=[wintypes.HWND,wintypes.UINT,wintypes.WPARAM,wintypes.LPARAM]
    return handles[0]


def resources(process):
    result = []
    for item in [process] + process.children(recursive=True):
        try:
            memory = item.memory_info()
            result.append({"pid": item.pid, "name": item.name(), "handles": item.num_handles(), "rss": memory.rss, "private": memory.private})
        except psutil.Error:
            pass
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", default="target/playground/amigo-app.exe")
    parser.add_argument("--seconds", type=float, default=3)
    parser.add_argument("--soak", type=int, default=300)
    parser.add_argument("--models", default="cube,sphere,suzanne,multi")
    parser.add_argument("--looks", default="comic-ink,pencil-study")
    parser.add_argument("--sizes", default="640x360,960x540,1280x720")
    parser.add_argument("--modes", default="native_gpu,local_rgba,jpeg")
    parser.add_argument("--layers", default="4,8,16", help="ordered layer-stack sizes measured per case")
    parser.add_argument("--output", default="target/viewport-benchmark.jsonl")
    parser.add_argument("--skip-matrix", action='store_true')
    parser.add_argument("--resume", action='store_true', help="append results and skip already valid matrix cases")
    args = parser.parse_args()
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    completed = set()
    if args.resume and output.exists():
        for line in output.read_text(encoding="utf-8").splitlines():
            try:
                record = json.loads(line)
                if record.get("kind") == "case" and record.get("valid"):
                    completed.add(tuple(record.get(key) for key in ("model", "look", "layers", "size", "mode", "scenario")))
            except json.JSONDecodeError:
                continue
    probe = socket.socket(); probe.bind(("127.0.0.1", 0)); port = probe.getsockname()[1]; probe.close()
    env = os.environ.copy()
    env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = f"--remote-debugging-port={port} --remote-debugging-address=127.0.0.1 --remote-allow-origins=http://localhost"
    env["WEBVIEW2_USER_DATA_FOLDER"] = str(output.with_suffix(f".webview-{os.getpid()}").resolve())
    env["AMIGO_NPR_BENCHMARK"] = "1"
    started = time.monotonic()
    result_mode = "a" if args.resume else "w"
    with output.with_suffix(".log").open("w", encoding="utf-8") as log, output.open(result_mode, encoding="utf-8") as results:
        process = subprocess.Popen([args.executable, "--hosted", "--mod", "npr-playground", "--scene", "gallery"], env=env, stdout=log, stderr=log, creationflags=subprocess.CREATE_NO_WINDOW)
        owner = psutil.Process(process.pid)
        try:
            deadline = time.monotonic() + 45
            while True:
                try:
                    pages = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list"))
                    page = next(page for page in pages if "tauri.localhost" in page["url"])
                    client = Client(page["webSocketDebuggerUrl"])
                    break
                except (OSError, StopIteration):
                    if time.monotonic() >= deadline: raise
                    time.sleep(.1)
            client.until("document.querySelector('.hud')?.textContent.startsWith('Native GPU')", 30)
            results.write(json.dumps({"kind":"cold_start","seconds":time.monotonic()-started,"resources":resources(owner)})+"\n");results.flush()
            client.js(MEASURE)
            native_hwnd=find_native(owner)
            for model in ([] if args.skip_matrix else args.models.split(',')):
                client.model(model)
                for look in args.looks.split(','):
                    client.look(look)
                    for layer_count in args.layers.split(','):
                        layer_count = int(layer_count)
                        client.layers(layer_count)
                        for size in args.sizes.split(','):
                            width,height=map(int,size.split('x'))
                            client.js(f"(()=>{{const v=document.querySelector('.viewport');v.style.flex='none';v.style.width='{width}px';v.style.height='{height}px';}})()")
                            for mode in args.modes.split(','):
                                client.mode(mode)
                                for scenario in ["orbit","spin","pause"]:
                                    key = (model, look, layer_count, size, mode, scenario)
                                    if key in completed:
                                        continue
                                    context = {"model":model,"look":look,"layers":layer_count,"size":size,"mode":mode,"scenario":scenario}
                                    results.write(json.dumps({"kind":"case_start", **context}) + "\n"); results.flush()
                                    try:
                                        record={"kind":"case",**context,**measure(client,scenario,args.seconds,native_hwnd)}
                                        for retry in range(2):
                                            if record['valid']:break
                                            record={**record,**measure(client,scenario,args.seconds,native_hwnd),'retry':retry+1}
                                    except (websocket.WebSocketException, OSError, ConnectionError) as error:
                                        record={"kind":"error",**context,"error_type":type(error).__name__,"error":str(error),"resources":resources(owner)}
                                        results.write(json.dumps(record)+"\n"); results.flush(); print(json.dumps(record), flush=True)
                                        raise SystemExit(2)
                                    results.write(json.dumps(record)+"\n");results.flush()
                                    print(json.dumps({key:value for key,value in record.items() if key not in ['raw','pointer_events']}),flush=True)
            if args.soak:
                # Resource measurements must not retain every frame in the test
                # harness. The matrix above already wrote its raw frame samples.
                client.js("__measure.enabled=false;__measure.frames=[];__measure.inputs={};__pointerEvents=[]")
                client.model('cube');client.look('comic-ink');client.checkbox('Enable spinning animation',True)
                client.js("(()=>{const v=document.querySelector('.viewport');v.style.flex='none';v.style.height='720px';})()")
                start=time.monotonic();iteration=0
                while time.monotonic()-start<args.soak:
                    client.mode(['native_gpu','local_rgba','jpeg'][iteration%3])
                    client.js(f"document.querySelector('.viewport').style.width='{[640,960,1280][iteration%3]}px'")
                    time.sleep(max(0,min(10,args.soak-(time.monotonic()-start))))
                    record={"kind":"soak","seconds":time.monotonic()-start,"resources":resources(owner),"hud":client.js("document.querySelector('.hud').textContent"),"error":client.js("document.querySelector('.presentation-error')?.textContent")}
                    results.write(json.dumps(record)+"\n");results.flush();print(json.dumps(record),flush=True)
                    iteration+=1
                # In the single-window lifecycle, closing Tauri also ends the
                # windowless engine. No hidden process should be left running.
                ctypes.windll.user32.GetAncestor.restype=wintypes.HWND
                parent=ctypes.windll.user32.GetAncestor(native_hwnd,2)
                ctypes.windll.user32.PostMessageW(parent,0x10,0,0)
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:pass
                record={'kind':'close','engine_exited':process.poll() is not None}
                results.write(json.dumps(record)+'\n');results.flush();print(json.dumps(record),flush=True)
                assert record['engine_exited'], 'Engine remained alive after closing Tauri'
        finally:
            try:children=owner.children(recursive=True)
            except psutil.NoSuchProcess:children=[]
            for child in children:
                try: child.kill()
                except psutil.Error: pass
            if process.poll() is None:process.kill()
            process.wait()


if __name__ == '__main__':
    main()
