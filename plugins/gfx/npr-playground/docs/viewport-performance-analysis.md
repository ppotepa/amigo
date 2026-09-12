# NprPlayground — analiza wydajności viewportu

Data: 2026-09-11. Historyczny punkt odniesienia sprzed optymalizacji runtime.
Aktualna implementacja i pomiary Tauri: [viewport-validation.md](viewport-validation.md).

## Wniosek

Problem kilku FPS da się odtworzyć przed WebSocketem i Tauri. W buildzie dev
obracany cube przy 1280×720 potrzebuje średnio 210,09 ms na przygotowanie JPEG.
Konwersja RGBA→RGB i JPEG zajmują łącznie 207,26 ms (98,7% tego czasu).
Release skraca całość do 19,77 ms, ale JPEG nadal kosztuje 17,32 ms.
Drugi niezależny problem to generowanie NPR dla większych siatek: Suzanne
z pencil-study potrzebuje około 51–58 ms samej ekstrakcji podczas orbitowania.

Najpierw potrzebne są optymalizowany profil uruchamiania, szybszy encoder oraz
naprawa invalidacji cache i inputu. Migracja na video nie usuwa kosztu ekstrakcji.

## Metoda i granice pomiaru

- Maszyna: Intel Core i5-12600K; release wybrał RTX 3070 Ti, backend Vulkan.
- Wersje z lockfile: image 0.25.10, wgpu 29.0.1.
- Sonda odtwarza etapy start_worker: rebuild_with_delta, packet/composition,
  render_frame_request, read_rgba8_blocking, tę samą konwersję RGB i JPEG Q92.
- Osobny proces, jeden renderer/target, bez otwierania Winit i WebView.
- Dwa rozgrzewkowe wywołania i średnia z siedmiu kolejnych na scenariusz.
- Standard: Settings::for_scene(false), jeden wybrany model lub pusta scena,
  paused=true, sketch_paused=true. Domyślny motion.mode pozostaje
  redraw-continuously. Orbit zmienia yaw o 2° na wywołanie.
- Pencil: rzeczywiste authored comic-ink + pencil-study, rozwiązywane przez
  resolve_look, kamera distance=5. Nie jest to pomiar dowolnego looka użytkownika.
- Osobna próba zmienia motion.mode na stable przy tej samej pauzie szkicu.
- Render/submit mierzy czas CPU wywołania renderera. Readback obejmuje oczekiwanie
  na wcześniejszą pracę GPU oraz kopiowanie, nie jest czystym czasem transferu GPU.
- Wynik 1000 / czas klatki to przepustowość sondy, nie zmierzony FPS w Tauri.
  Brakuje tu harmonogramu hosta, konkurencji dwóch widoków, transportu, WebView
  i prezentacji monitora. Nie wyznaczam p95 na podstawie siedmiu próbek.
- Statyczne scenariusze wymuszają kolejne wywołania w celu pomiaru kosztu.
  Produkcyjny stream może poprawnie przestać wysyłać klatki w spoczynku.

Sonda i surowe wyniki pozostają w ignorowanym target:

- target/viewport_profile_probe.rs
- target/viewport-profile-dev.jsonl
- target/viewport-profile-release.jsonl
- target/viewport-profile-pencil-release.jsonl
- target/viewport-profile-pencil-stable-release.jsonl

Sonda była tymczasowym przykładem crates/runtime/bundles/examples/viewport_profile_probe.rs;
po pomiarach usunięto ją z drzewa źródeł. Powtórzenie wymaga skopiowania zachowanej
sondy pod tę ścieżkę, a następnie uruchomienia:
```powershell
rtk cargo run -p amigo-runtime-bundles --example viewport_profile_probe
rtk cargo run --release -p amigo-runtime-bundles --example viewport_profile_probe
rtk cargo run --release -p amigo-runtime-bundles --example viewport_profile_probe -- --pencil
rtk cargo run --release -p amigo-runtime-bundles --example viewport_profile_probe -- --pencil --stable
```

## Wyniki

Cube podczas orbitowania, milisekundy:

| Etap | dev 640×360 | dev 1280×720 | release 1280×720 |
|---|---:|---:|---:|
| Ekstrakcja NPR | 0,290 | 0,270 | 0,073 |
| Packet/composition | 0,025 | 0,026 | 0,008 |
| Render/submit CPU | 1,155 | 1,199 | 0,349 |
| Readback: wait + copy | 0,652 | 1,332 | 1,334 |
| RGBA→RGB | 15,780 | 64,176 | 0,685 |
| JPEG Q92 | 35,936 | 143,088 | 17,322 |
| Razem | 53,838 | 210,092 | 19,771 |
| 1000 / czas klatki | 18,6 | 4,8 | 50,6 |

Pusta scena dev 1280×720: 210,55 ms. Sphere orbit dev: 206,28 ms.
To silny dowód, że kilka FPS prostych modeli nie wymaga problemu z geometrią
ani powolnego dekodowania w WebView. Release cube 640×360: 5,75 ms.

Pencil-study, release 1280×720:

| Model/scenariusz | Ekstrakcja NPR | JPEG | Całość |
|---|---:|---:|---:|
| Cube orbit | 0,25 ms | 22,72 ms | 25,08 ms |
| Cylinder orbit | 0,80 ms | 22,67 ms | 25,50 ms |
| Sphere orbit | 0,47 ms | 23,35 ms | 26,65 ms |
| Suzanne orbit | 51,16 ms | 22,66 ms | 78,76 ms |
| Suzanne nieruchoma, sketch_paused, domyślny motion.mode | 52,05 ms | 22,67 ms | 79,73 ms |
| Suzanne nieruchoma, sketch_paused, motion.mode=stable | 0,33 ms | 22,57 ms | 27,43 ms |

Suzanne używała 3936 trójkątów źródłowych i 15744 w smooth proxy.
Zmiana mode na stable nie naprawia kosztu orbitowania: w osobnej próbie ekstrakcja
nadal wyniosła 58,21 ms. Eliminuje zbędną przebudowę statycznego obrazu.

## Audyt ścieżki

### 1. Profil builda i kodowanie — potwierdzone główne ograniczenie prostych scen

READ crates/runtime/bundles/src/npr_playground_viewport.rs, start_worker:
po read_rgba8_blocking następują chunks_exact(4).flat_map(...).collect()
i image::JpegEncoder jakości 92. Nowe bufory RGBA, RGB i JPEG powstają co klatkę.
Całość jest sekwencyjna w osobnym workerze; nie blokuje bezpośrednio głównego
wątku na readbacku, ale ogranicza przepustowość viewportu.

READ Cargo.toml oraz .cargo/config.toml: brak profilu optymalizującego dev.
Vite buduje już frontend produkcyjnie; to nie zastępuje optymalizacji Rust.
[Cargo domyślnie używa opt-level=0 dla dev](https://doc.rust-lang.org/cargo/reference/profiles.html).

### 2. Cache i częstotliwość szkicu — błąd potwierdzony A/B

READ plugins/gfx/npr-playground/src/render/mod.rs, rebuild_internal:
continuous_redraw sprawdza motion.mode, ale nie sketch_paused ani faktyczną
zmianę epoki wariantu kreski. Dopiero później dla zegara wariantów stosuje effective
motion=Stable przy pauzie. Przebudowa pakietu zdążyła już zostać wymuszona.

READ crates/engine/render-npr/src/temporal.rs: domyślnie redraw-continuously
z redraw_hz=8. Zegar wariantów ogranicza epoch do tej częstotliwości, ale extractor
może generować cały pakiet przy każdym wywołaniu renderowym. Fade/temporal
continuity powinny działać oddzielnie od kosztownej generacji źródeł kreski.

### 3. Dwa widoki i praca CPU NPR — potwierdzona struktura, częściowo zmierzony koszt

READ plugins/gfx/npr-playground/src/plugin.rs, npr_playground_extract:
Winit wywołuje własne rebuild_with_delta w RenderExtract. Worker companionu
tworzy drugi NprPlaygroundRenderService oraz osobny WGPU device/renderer.
Oba widoki mają własne cache, ekstrakcję i render. Ciężki native RenderExtract
ogranicza także częstotliwość hostowych PostUpdate obsługujących input i nowe jobs.

Pakiety zawierają współrzędne ekranowe, dlatego nie można bezwarunkowo współdzielić
gotowego packetu pomiędzy różnymi rozmiarami, kamerami i stanem temporalnym.
Można współdzielić niezmienną geometrię/topologię/proxy; reuse gotowego packetu
wymaga identycznego pełnego klucza widoku.

READ crates/engine/render-npr/src/frame.rs: SurfaceDirectionField::build jest
wywoływany podczas budowy packetu i ponownie w emit_surface_hatching.
READ render/mod.rs: globalny klucz Settings unieważnia wszystkie obiekty.
Są też pełne kopie source/output/commands; gallery budget klonuje kreski nawet,
gdy ostatecznie niczego nie odrzuca. To kandydaci do pomiaru podetapów;
nie przypisuję im całych 51 ms bez dodatkowej instrumentacji.

### 4. Readback i upload — poprawić po największych kosztach CPU

READ crates/engine/render-wgpu/src/backend/surface.rs, read_rgba8_blocking:
nowy staging buffer, copy, map_async, PollType::Wait, recv i kopiowanie
każdej klatki. Brak puli i nakładania etapów render/readback/encode.

READ crates/engine/render-wgpu/src/renderer/service/render/world.rs,
render_npr_commands, oraz renderer/npr.rs: konwersja vertices i create_buffer_init
per batch per klatka. Nawet reuse CPU packetu nie oznacza reuse uploadu.
Pomiar prostego cube nie uzasadnia rozpoczynania całej optymalizacji od GPU.

[WGPU 29 wymaga zakończenia użycia bufora przez GPU przed mapowaniem](https://docs.rs/wgpu/29.0.0/wgpu/struct.Buffer.html);
asynchroniczny readback potrzebuje osobnych slotów i jawnego lifecycle.

### 5. Backpressure kończy się przed klientem

READ crates/engine/playground-api/src/viewport.rs oraz
crates/runtime/bundles/src/npr_playground_viewport.rs:
jobs rendezvous (0) i frame queue (2) już ograniczają kolejki aplikacyjne.
Resize targetu już zachodzi tylko przy zmianie rozmiaru. Nie trzeba ponownie
implementować tych mechanizmów.

READ crates/runtime/bundles/src/playgrounds.rs, serve_client:
jedna pętla wykonuje blocking read (timeout 5 ms), control send i binary frame send
(timeout write 100 ms). Wolny write może opóźniać obsługę wejścia; nie ma pomiaru
tych opóźnień. Nie jest to dowód, że sieć powoduje obecne 4–5 FPS.

sequence/revision/size z PlaygroundViewportFrame nie trafiają do wiadomości
binarnej: wysyłany jest sam JPEG. Nie ma ACK prezentacji ani kontroli wieku
klatki w buforach TCP/przeglądarki. Queue(2) nie zapewnia bounded latency całej drogi.
[WebSocket API nie ma automatycznego backpressure odbioru](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
[bufferedAmount mierzy wyłącznie kolejkę wysyłania](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/bufferedAmount).

### 6. Canvas i metryki — potwierdzone problemy kodu, koszt niezmierzony

READ playground-client/src/App.svelte: canvas.width i canvas.height są ustawiane
przy każdej klatce, nawet bez resize; drawImage działa bezpośrednio w callbacku
decode, a HUD aktualizuje reaktywny stan co klatkę. FPS jest odwrotnością jednego
odstępu między callbackami, nie stabilnym pomiarem prezentacji.

[Przypisanie canvas.width resetuje kontekst również dla tej samej wartości](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/width).
READ session.ts: decoder odrzuca gotowy bitmap, jeśli czeka już następny Blob.
Przy ciągłym przeciążeniu może stale dekodować i niczego nie prezentować.
Potrzebne są osobne sloty latest received / latest decoded oraz prezentacja w rAF.

### 7. Sterowanie i synchronizacja — osobna przyczyna odczuwanej nierówności

READ App.svelte i camera-input.ts:
LatestNavigationQueue zastępuje względne dx/dy/wheel. Dziesięć ruchów po 5 px
może zamienić się w jeden ruch 5 px. Należy sumować delty tego samego gestu albo
wysyłać skumulowany cel względem początku gestu; latest-wins nadaje się do
absolutnego stanu i obrazów, nie do dowolnego usuwania względnego inputu.

Próg 4 px sprawdzany jest ponownie po każdym ruchu zamiast raz dla rozpoczęcia
dragu. Brakuje korelacji accepted/delta z request_id. Dowolna delta zwalnia
navigationInFlight; formularz może wysłać akcję podczas nawigacji z tą samą
base_revision. Dodatkowo pan miesza delty CSS z wysokością fizycznego canvasu.
Plan musi objąć także blur/cancel i synchronizację po odrzuceniu rewizji.

READ playground-api/src/host.rs oraz plugin playground.rs:
sprawdzanie rewizji i polling tworzą pełne snapshoty. Model descriptors wraz
z dużymi SVG thumbnails są kopiowane/serializowane ponownie; zmiana kamery
odsyła cały top-level npr, nie deltę pojedynczego pola. Undo zapisuje każdą
akcję nawigacji, zamiast jednego wpisu na gest. Te koszty wymagają pomiaru w hoście.

## Plan wdrożenia w kolejności

Każdy etap kończy się tym samym scenariuszem pomiarowym. Nie zakładać, że samo
zwiększenie limitu 30/60 zwiększy rzeczywistą przepustowość.

Ścieżki tabeli względem repozytorium: playground-api i render-wgpu/render-npr
oznaczają crates/engine/<nazwa>; runtime/bundles oznacza crates/runtime/bundles;
playground-client i plugin render/mod.rs / playground.rs należą do
plugins/gfx/npr-playground (pliki Rust pod src).

| Etap / operacja | Dokładny zakres i cel | Walidacja | Granica |
|---|---|---|---|
| P0 ADD metryki | playground-api/src/viewport.rs: frame header i timestamps; runtime/bundles/src/npr_playground_viewport.rs: czasy etapów, drops; playground-client/src/session.ts i App.svelte: receive/decode/draw/ACK, HUD co 0,5–1 s | cargo test -p amigo-playground-api; cargo check -p amigo-runtime-bundles; npm run check; scenariusze p50/p95 | Neutralne pola, bez typów Tauri/WGPU w API; rozróżnić zegary procesów |
| P0 MODIFY profil | Cargo.toml: nazwany optymalizowany profil developerski lub sprawdzone overrides image, runtime-bundles, render-npr, plugin; dokumentacja komendy startowej | benchmark dev/optimized/release na identycznym obrazie | Nie utożsamiać builda Vite z buildem Rust; mierzyć narzut kompilacji |
| P0 MODIFY invalidację | plugins/gfx/npr-playground/src/render/mod.rs: najpierw efektywna polityka pauzy/epoch, potem rebuild; lekki fade niezależnie | cargo test -p amigo-npr-playground-plugin; test liczby rebuildów i temporal/golden | Nie wyłączać rysunkowej animacji użytkownikowi; nie zmieniać authored looka |
| P1 MODIFY input | playground-client/src/camera-input.ts i App.svelte: akumulacja, jeden próg gestu, spójne jednostki, request_id, jedna serializacja mutacji; plugin playground.rs: historia per gesture | npm test; test burst input z delayed ACK i obcą deltą; plugin tests; rzeczywisty drag w Tauri | Nie omijać walidacji base_revision; nie robić kamery niezależnej od hosta |
| P1 MODIFY encode | runtime/bundles/src/npr_playground_viewport.rs: reuse buforów, encoder przyjmujący RGBA/stride albo jedna szybka konwersja; porównać image i SIMD JPEG przy Q92/85/75 | benchmark ms i bytes + wizualne porównanie konturów, koloru, hatch i papieru | Zachować high-quality capture; nie obiecywać proporcjonalnego zysku z samego Q75 |
| P1 MODIFY odbiór | playground-client/src/session.ts i App.svelte: resize tylko przy zmianie, cached context, latest decoded w rAF, ImageBitmap.close, brak głodzenia prezentacji | test decoder producer>consumer, resize i memory soak; browser timing | Najpierw zmierzyć; Worker/OffscreenCanvas dopiero gdy decode/draw blokuje UI |
| P2 MODIFY stream | runtime/bundles/src/playgrounds.rs i playground-api/src/viewport.rs: frame IDs/generation, 1–2 kredyty/ACK, priorytet control, event-driven socket, brak produkowania starych klatek | delayed reader, zatrzymany WebView, resize generation, scene close, bounded memory | Nie replayować jednorazowego tokenu; zmiana protokołu wymaga jawnej wersji |
| P2 MODIFY readback | render-wgpu/src/backend/types.rs i surface.rs: 2–3 sloty staging; submit/poll-completed, pula rozmiarów; bridge: encoder w osobnym workerze | cargo check -p amigo-render-wgpu; test reuse/unmap/resize/close; GPU timing | Brak PollType::Wait w ścieżce streamu; blokująca ścieżka pozostaje dla eksportu/testów |
| P2 MODIFY cache NPR | plugin render/mod.rs; render-npr/src/surface.rs, frame.rs, contour.rs: cache per object/surface revision, wspólne field/normals, osobno view-dependent geometry | cargo test -p amigo-render-npr; plugin tests; NPR golden | Chronić obecne zmiany humanizacji; zachować stable stroke IDs, kolejność i deterministykę |
| P2 MODIFY upload | render-wgpu/src/renderer/npr.rs i service/render/world.rs: reuse vertex/index buffers, dirty ranges i cache packet generation | cargo check -p amigo-render-wgpu; render golden/stats | Nie zmieniać MeshDrawCommand ani wprowadzać interpretacji looków w backendzie |
| P3 MODIFY adaptację | bridge i playground-api viewport request: render scale ze średnich/p95, szybka redukcja i wolny powrót; np. start 960×540 przy ruchu, pełna jakość po uspokojeniu | test aspect/DPR/picking i sekwencje motion/idle/capture | Limity kreski w domenie, rozdzielczość transferu w bridge; bez nadpisywania authored state |
| P3 MODIFY host/state | plugin playground.rs i playground-api host.rs: tania revision(), cache metadata/models, drobniejsze delty; współdzielenie immutable powierzchni między widokami | test wersjonowania, asset lifecycle, undo i dwa różne viewporty | Winit nadal widoczny; bez zatrzymywania symulacji/skryptów po utracie focusu |

P0–P1 mają największy przewidywany wpływ na prosty model. Dla Suzanne etap cache
NPR trzeba prowadzić wcześnie: nawet idealny transport nie zmieści 51 ms
ekstrakcji w budżecie 16,7 ms.

Widget nad viewportem zachowuje tylko tryb kamery i przełącznik spinning animation,
zgodnie z ustalonym zakresem. Szczegółowe metryki należą do Diagnostics; podczas
zwykłej pracy wystarcza dyskretny HUD ze stabilnym FPS, rozdzielczością lub Idle.
Spinning modelu i animacja wariantów szkicu pozostają osobnymi stanami domeny.

## Wybór technologii streamowania

| Wariant | Ocena dla obecnego problemu |
|---|---|
| WebSocket + szybszy JPEG | Pierwszy wybór: istniejący przepływ, niezależne klatki, łatwe odrzucanie, testowalna jakość. SIMD encoder jest kandydatem, nie zmierzonym jeszcze rozwiązaniem. |
| Raw RGBA | Przy 1280×720×60 to 221 MB/s samych pikseli (~211 MiB/s), dodatkowo IPC/kopie. Sensowny wariant diagnostyczny, nie automatyczne optimum. |
| H.264/VP9 + WebCodecs lub WebRTC | Kolejny eksperyment dla 1080p/60 lub wielu klientów, jeśli JPEG po optymalizacji nadal dominuje. Potrzebny rzeczywisty encoder po stronie hosta, negocjacja wsparcia, keyframes, color pipeline i pomiar latency. |
| Native GPU texture sharing | Osobny projekt platformowy; wykracza poza ustalone v1. Aktualny Vulkan/Windows wymaga audytu interop. |
| Klatki przez Tauri events/base64 | Brak uzasadnienia dla tej migracji: obecny klient już odbiera binarnie bez tego pośrednictwa. |

[libjpeg-turbo opisuje wsparcie SIMD i formatów RGBA/BGRA](https://github.com/libjpeg-turbo/libjpeg-turbo);
przed przyjęciem zależności trzeba zmierzyć build Windows, jakość i koszt FFI.
[WebCodecs udostępnia niskopoziomowe kodowanie/dekodowanie](https://developer.mozilla.org/en-US/docs/Web/API/WebCodecs_API);
nie dostarcza automatycznie transportu, hostowego encodera ani zero-copy z WGPU.
[OffscreenCanvas pozwala przenieść render canvasu do workera](https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvas);
nie przyspiesza kodowania JPEG w Rust.

## Kryteria odbioru optymalizacji

To cele, nie wyniki obecnej implementacji:

- Na sprzęcie referencyjnym: po rozgrzaniu stabilne ~30 FPS spinning prostych
  modeli i cel 60 FPS orbitowania (p95 odstępu prezentacji <=20 ms).
- Cel p95 input→widoczna zmiana <=80 ms; brak wielosekundowego doganiania.
  Mierzyć od inputu do ACK narysowania konkretnego frame/input sequence,
  z korektą offsetu zegarów. Callback draw nie dowodzi fizycznego scanoutu.
- Nieruchoma, zatrzymana scena: po fade brak ciągłej ekstrakcji/enkodowania,
  HUD pokazuje Idle zamiast mylącego stale zapamiętanego FPS.
- Producer szybszy od konsumenta: bounded memory, najwyżej ustalona liczba
  klatek in flight, brak głodzenia prezentacji i blokowania formularzy.
- Sprawdzić cube/cylinder/sphere, Suzanne, kilka obiektów; comic-ink/pencil-study;
  640×360, 960×540, 1280×720, DPR 1/2; orbit/pan/wheel/spin/stable/redraw.
- Osobno zimny start (thumbnail/import), test po rozgrzaniu >=30 s i soak >=60 s.
- Porównanie Winit bez companionu i z companionem; pomiary CPU/GPU, dropped frames
  per stage, decode/draw, bajty/s, host tick i p50/p95 frame age.
- NPR golden renderu przed kompresją nadal przechodzą. JPEG/video mają oddzielne
  porównanie wizualne; nie podmieniać goldenów, aby ukryć regresję jakości.

Po odpowiednich zmianach: cargo test -p amigo-playground-api,
cargo test -p amigo-npr-playground-plugin, cargo test -p amigo-render-npr,
cargo check -p amigo-runtime-bundles, npm run check, npm test, npm run build.
Dla zmian wspólnych kontraktów także cargo check -p amigo-app, cargo check
-p amigo-playground-tauri oraz istniejące testy NPR w amigo-app.
Testy przepustowości wykonywać w ustalonym profilu optymalizowanym, nie jako
niestabilne asercje czasowe w zwykłych unit testach.

## Stan prac po audycie

Wykonano cztery przebiegi sondy, odczyt źródeł i dokumentacji API. Pierwsza próba
sondy odrzuciła niepoprawną selekcję pustej sceny; naprawiono wyłącznie sondę,
po czym przebiegi zakończyły się sukcesem. Nie zmieniono produkcyjnego runtime,
frontendowego sterowania ani renderu. Brak pomiaru rzeczywistej prezentacji Tauri,
flamegraphu 51 ms ekstrakcji i benchmarku alternatywnego encodera jest jawny;
należy je wykonać w P0/P1, zanim uznamy docelowe 30/60 FPS za osiągnięte.
