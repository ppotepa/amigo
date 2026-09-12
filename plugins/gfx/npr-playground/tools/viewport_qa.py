"""Visual/lifecycle checks against the real companion (Windows, local CDP only)."""
import base64
import ctypes
from ctypes import wintypes
import io
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import urllib.request

from PIL import Image, ImageChops, ImageStat
import psutil
from viewport_benchmark import Client, find_native


def print_window(hwnd):
    user, gdi = ctypes.windll.user32, ctypes.windll.gdi32
    user.GetDC.restype = wintypes.HDC
    gdi.CreateCompatibleDC.argtypes = [wintypes.HDC]; gdi.CreateCompatibleDC.restype = wintypes.HDC
    gdi.CreateCompatibleBitmap.argtypes = [wintypes.HDC, ctypes.c_int, ctypes.c_int]; gdi.CreateCompatibleBitmap.restype = wintypes.HBITMAP
    gdi.SelectObject.argtypes = [wintypes.HDC, wintypes.HANDLE]; gdi.SelectObject.restype = wintypes.HANDLE
    user.PrintWindow.argtypes = [wintypes.HWND, wintypes.HDC, wintypes.UINT]
    gdi.GetDIBits.argtypes = [wintypes.HDC,wintypes.HBITMAP,wintypes.UINT,wintypes.UINT,ctypes.c_void_p,ctypes.c_void_p,wintypes.UINT]
    gdi.DeleteObject.argtypes = [wintypes.HANDLE]; gdi.DeleteDC.argtypes = [wintypes.HDC]
    user.ReleaseDC.argtypes = [wintypes.HWND,wintypes.HDC]
    class Header(ctypes.Structure):
        _fields_=[('size',wintypes.DWORD),('width',wintypes.LONG),('height',wintypes.LONG),('planes',wintypes.WORD),('bits',wintypes.WORD),('compression',wintypes.DWORD),('image_size',wintypes.DWORD),('x',wintypes.LONG),('y',wintypes.LONG),('colors',wintypes.DWORD),('important',wintypes.DWORD)]
    rect=wintypes.RECT();user.GetClientRect(hwnd,ctypes.byref(rect))
    width,height=rect.right,rect.bottom
    dc=user.GetDC(hwnd); memory=gdi.CreateCompatibleDC(dc);bitmap=gdi.CreateCompatibleBitmap(dc,width,height);previous=gdi.SelectObject(memory,bitmap)
    try:
        if not user.PrintWindow(hwnd,memory,2):raise RuntimeError('PrintWindow failed')
        header=Header(ctypes.sizeof(Header),width,-height,1,32,0,width*height*4,0,0,0,0)
        pixels=ctypes.create_string_buffer(width*height*4)
        if not gdi.GetDIBits(memory,bitmap,0,height,pixels,ctypes.byref(header),0):raise RuntimeError('GetDIBits failed')
        return Image.frombytes('RGB',(width,height),pixels.raw,'raw','BGRX')
    finally:
        gdi.SelectObject(memory,previous);gdi.DeleteObject(bitmap);gdi.DeleteDC(memory);user.ReleaseDC(hwnd,dc)


def launch(output, env, scene='gallery'):
    log=output.with_suffix('.log').open('w',encoding='utf-8')
    process=subprocess.Popen(['target/playground/amigo-app.exe','--hosted','--mod','npr-playground','--scene',scene],env=env,stdout=log,stderr=log,creationflags=subprocess.CREATE_NO_WINDOW)
    return process,psutil.Process(process.pid),log


def connect(port):
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        try:
            page=next(page for page in json.load(urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list')) if 'tauri.localhost' in page['url'])
            return Client(page['webSocketDebuggerUrl'])
        except (OSError,StopIteration):time.sleep(.1)
    raise TimeoutError('companion did not start')


def close(process,owner,log):
    try:
        native=find_native(owner)
        ctypes.windll.user32.GetAncestor.restype=wintypes.HWND
        parent=ctypes.windll.user32.GetAncestor(native,2)
        ctypes.windll.user32.PostMessageW(parent,0x10,0,0)
        deadline=time.monotonic()+5
        while process.poll() is None and time.monotonic()<deadline:time.sleep(.05)
    except (RuntimeError,psutil.Error):pass
    exited=process.poll() is not None
    try:children=owner.children(recursive=True)
    except psutil.NoSuchProcess:children=[]
    for child in children:
        try:child.kill()
        except psutil.Error:pass
    if process.poll() is None:process.kill()
    process.wait();log.close()
    return exited


def visible_app_windows(owner):
    pids={owner.pid,*(child.pid for child in owner.children(recursive=True) if child.name()=='amigo-app.exe')}
    windows=[]
    callback_type=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
    def visit(hwnd,_):
        pid=wintypes.DWORD();ctypes.windll.user32.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
        if pid.value in pids and ctypes.windll.user32.IsWindowVisible(hwnd):
            rect=wintypes.RECT();ctypes.windll.user32.GetClientRect(hwnd,ctypes.byref(rect))
            # Winit's internal event target has WS_VISIBLE but a zero-sized
            # client area; it is not an application window shown to the user.
            if rect.right>0 and rect.bottom>0:windows.append(pid.value)
        return True
    ctypes.windll.user32.EnumWindows(callback_type(visit),0)
    return windows


def main():
    output=Path('target/viewport-qa');output.mkdir(exist_ok=True)
    probe=socket.socket();probe.bind(('127.0.0.1',0));port=probe.getsockname()[1];probe.close()
    env=os.environ.copy();env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']=f'--remote-debugging-port={port} --remote-debugging-address=127.0.0.1 --remote-allow-origins=http://localhost'
    env['WEBVIEW2_USER_DATA_FOLDER']=str((output/f'webview-{os.getpid()}').resolve())
    process,owner,log=launch(output/'first',env)
    report={}
    try:
        client=connect(port);client.until("document.querySelector('.hud')?.textContent.includes('GPU')",30)
        report['default']=client.js("document.querySelector('.presentation select').value")
        windows=visible_app_windows(owner)
        report['gallery_has_only_tauri_window']=len(windows)==1 and windows[0]!=owner.pid
        client.js("window.__qaHeader=null;window.__qaNavigation=0;window.addEventListener('playground-frame-presented',e=>window.__qaHeader=e.detail);window.addEventListener('playground-input-sent',e=>{if(e.detail.kind==='navigate')__qaNavigation++;})")
        client.model('cube');client.look('pencil-study');client.tab('Motion');client.checkbox('Pause sketch',True)
        client.js("(()=>{const v=document.querySelector('.viewport');v.style.flex='none';v.style.width='640px';v.style.height='360px';})()")
        native=find_native(owner)
        for mode in ['native_gpu','local_rgba','jpeg']:
            if mode=='local_rgba':
                client.js("window.__rgba=null;window.addEventListener('playground-frame-presented',e=>{if(e.detail.mode==='local_rgba'){const c=document.querySelectorAll('canvas')[1],g=c.getContext('webgl2'),p=new Uint8Array(c.width*c.height*4);g.readPixels(0,0,c.width,c.height,g.RGBA,g.UNSIGNED_BYTE,p);window.__rgba={size:[c.width,c.height],pixels:Array.from(p)};}},{once:false})")
            client.mode(mode);time.sleep(.8)
            if mode=='native_gpu':
                picture=print_window(native)
            elif mode=='local_rgba':
                data=client.js('__rgba');picture=Image.frombytes('RGBA',tuple(data['size']),bytes(data['pixels'])).transpose(Image.Transpose.FLIP_TOP_BOTTOM).convert('RGB')
            else:
                data=client.js("document.querySelector('canvas').toDataURL('image/png').split(',')[1]")
                picture=Image.open(io.BytesIO(base64.b64decode(data))).convert('RGB')
            picture.save(output/f'{mode}.png')
            report[mode]={'size':picture.size,'extrema':picture.getextrema(),'hud':client.js("document.querySelector('.hud').textContent")}
        rgba=Image.open(output/'local_rgba.png');jpeg=Image.open(output/'jpeg.png')
        report['native_rgba_mean_absolute_error']=ImageStat.Stat(ImageChops.difference(Image.open(output/'native_gpu.png'),rgba)).mean
        report['jpeg_rgba_mean_absolute_error']=ImageStat.Stat(ImageChops.difference(rgba,jpeg)).mean
        report['selection']=client.js("document.querySelector('.outline-row.active')?.textContent")
        # Pause only this test's engine and fill its input queue. The companion
        # must survive and drain the ordered requests when the engine resumes.
        client.js("window.__qaSend=WebSocket.prototype.send;WebSocket.prototype.send=function(data){if(typeof data==='string'&&JSON.parse(data).type==='viewport'){window.__qaSocket=this;window.__qaViewport=data;}return __qaSend.call(this,data);}")
        client.mode('local_rgba')
        owner.suspend()
        try:
            client.js("for(let i=0;i<256;i++)__qaSocket.send(__qaViewport)")
            time.sleep(.4)
        finally:
            owner.resume()
        time.sleep(.5)
        client.mode('jpeg')
        report['full_input_queue_survives_and_recovers']=True
        client.js("WebSocket.prototype.send=__qaSend;delete window.__qaSocket;delete window.__qaViewport;delete window.__qaSend")
        client.mode('native_gpu')
        client.js("[...document.querySelectorAll('header button')].find(x=>x.textContent==='Help').click()")
        time.sleep(.2);report['modal_hides_native']=not bool(ctypes.windll.user32.IsWindowVisible(native))
        client.js("[...document.querySelectorAll('.dialog button')].find(x=>x.textContent==='Close').click()")
        time.sleep(.2);report['modal_restores_native']=bool(ctypes.windll.user32.IsWindowVisible(native))
        # Exercise overlapping configure/ready messages, then require a frame
        # from the final size and mode rather than relying on the previous HUD.
        generation=client.js('__qaHeader.generation')
        client.js("(async()=>{for(let i=0;i<18;i++){const v=document.querySelector('.viewport');v.style.width=(610+i*3)+'px';v.style.height=(340+i)+'px';if(i%3===0){const s=document.querySelector('.presentation select');s.value=['native_gpu','local_rgba','jpeg'][i/3%3];s.dispatchEvent(new Event('change',{bubbles:true}));}await new Promise(r=>setTimeout(r,24));}const v=document.querySelector('.viewport');v.style.width='640px';v.style.height='360px';})()")
        client.mode('native_gpu')
        client.until(f"__qaHeader?.mode==='native_gpu'&&__qaHeader.size[0]===640&&__qaHeader.size[1]===360&&__qaHeader.generation>{generation}")
        report['rapid_resize_and_switch']=True
        client.call('Emulation.setDeviceMetricsOverride',width=1600,height=1100,deviceScaleFactor=2,mobile=False)
        client.js("window.dispatchEvent(new Event('resize'))")
        for mode in ['native_gpu','local_rgba','jpeg']:
            client.mode(mode)
            client.until(f"__qaHeader?.mode==='{mode}'&&__qaHeader.size[0]===1280&&__qaHeader.size[1]===720")
        report['emulated_dpr2_all_modes']=True
        client.call('Emulation.clearDeviceMetricsOverride')
        client.js("window.dispatchEvent(new Event('resize'))")
        client.until("__qaHeader?.size[0]===640&&__qaHeader.size[1]===360")
        report['dpr1_restored']=True
        client.tab('Model')
        client.js("document.querySelector('input[aria-label=\"Search models\"]').focus()")
        before=client.js('__qaNavigation')
        for key in ['f','Home','r']:
            client.call('Input.dispatchKeyEvent',type='keyDown',key=key)
            client.call('Input.dispatchKeyEvent',type='keyUp',key=key)
        time.sleep(.2)
        report['form_shortcuts_do_not_navigate']=client.js('__qaNavigation')==before
        # Lose the WebGL context during a switch; JPEG remains a manual choice.
        client.mode('jpeg')
        client.js("document.querySelectorAll('canvas')[1].getContext('webgl2').getExtension('WEBGL_lose_context').loseContext()")
        time.sleep(.3)
        client.js("(()=>{const s=document.querySelector('.presentation select');s.value='local_rgba';s.dispatchEvent(new Event('change',{bubbles:true}));})()")
        time.sleep(.8)
        report['device_loss_error']=client.js("document.querySelector('.presentation-error')?.textContent")
        report['device_loss_preserves_jpeg']=client.js("document.querySelector('.hud').textContent.startsWith('JPEG')")
        client.mode('jpeg')
        report['preference']=client.js("localStorage.getItem('npr.presentation')")
    finally:
        (output/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
        close(process,owner,log)
    time.sleep(.3)
    process,owner,log=launch(output/'restored',env)
    try:
        client=connect(port)
        try:client.until("document.querySelector('.hud')?.textContent.startsWith('JPEG')",30)
        except TimeoutError:
            report['restored_error']=client.js("({hud:document.querySelector('.hud')?.textContent,error:document.querySelector('.presentation-error')?.textContent,stored:localStorage.getItem('npr.presentation'),chosen:document.querySelector('.presentation select')?.value})")
            (output/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
            raise
        report['restored_preference']=client.js("document.querySelector('.presentation select').value")
        for mode in ['native_gpu','local_rgba','jpeg']:client.mode(mode)
        report['restored_jpeg_can_switch_through_dx12_and_rgba']=True
        client.mode('local_rgba')
        client.js("document.querySelectorAll('canvas')[1].getContext('webgl2').getExtension('WEBGL_lose_context').loseContext()")
        client.until("!!document.querySelector('.presentation-error')")
        client.mode('native_gpu')
        report['active_local_device_loss_allows_manual_native']=True
    finally:report['closing_tauri_stops_engine']=close(process,owner,log)
    process,owner,log=launch(output/'cube',env,'cube')
    try:
        time.sleep(3);report['cube_has_no_companion']=not any(child.name()=='amigo-app.exe' for child in owner.children())
        report['cube_has_one_host_window']=visible_app_windows(owner)==[owner.pid]
    finally:close(process,owner,log)
    (output/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report))
    assert report['default']=='native_gpu'
    assert report['preference']==report['restored_preference']=='jpeg'
    assert report['native_rgba_mean_absolute_error']==[0,0,0]
    assert max(report['jpeg_rgba_mean_absolute_error'])<3
    assert all(value for value in report.values() if isinstance(value,bool)), report


if __name__=='__main__':main()
