# NPR: linie formy i stabilność rysunku podczas ruchu

Status: **specyfikacja do implementacji**, nie deklaracja gotowych funkcji.
Audyt źródeł i literatura: 2026-09-08. Nie wykonano w tej pracy nowego renderu
galerii ani pomiaru wydajności. Obserwacje wizualne pochodzą ze zgłoszenia użytkownika.

Dokument uszczegóławia [NPR: od modelu 3D do wiarygodnego rysunku](npr-drawing.md).
W zakresie reprezentacji powierzchni, tożsamości kresek, zachowania podczas ruchu
i sterowania tym zachowaniem niniejsza specyfikacja jest planem nadrzędnym.
Ogólny model tonu, narzędzia, papieru i depozycji pozostaje w tamtym dokumencie.
Nie tworzymy drugiego pipeline'u NPR ani konkurencyjnego zestawu profili.

Wzory oznaczone jako model projektowy, nazwy nowych typów, wartości początkowe
i pseudokod są propozycją dla Amigo. Nie są wynikami eksperymentu ani gotowym API.
Publikacje uzasadniają mechanizmy; nie dowodzą, że dowolne ich połączenie będzie
wyglądało jak rysunek wykonany przez człowieka.

## 1. Cel i znaczenie słowa „temporal”

Użytkownik chce wyłączać **przerysowywanie śladów przy obrocie modelu lub ruchu
kamery**. Obrót, orbitowanie, zoom i prawidłowa zmiana widoczności mają działać
dalej. Nie chodzi o zatrzymanie symulacji ani o odtwarzanie kolejności rysowania.

Drugi cel: organiczna forma, szczególnie obrzeże ucha Suzanne, ma być opisana
kilkoma sensownymi rodzinami linii, bez ujawniania triangulacji jako rusztowania.
Nie zakładamy konkretnej liczby kresek dla każdego ucha i każdego ujęcia.

### 1.1. Wymagane własności

1. Kreskowanie i prawdziwe załamania zachowują zakotwiczenie na powierzchni.
2. Sylwetka może przesuwać się po powierzchni, ale nie losuje całego gestu od nowa.
3. Brak ruchu i zmian danych nie powoduje samoczynnej zmiany wzoru śladów.
4. Zasłonięte fragmenty znikają zgodnie z aktualną geometrią, również bez fade.
5. Zmiana gęstości wyświetlania nie regeneruje wszystkich ścieżek i identyfikatorów.
6. Dodatkowe linie opisują formę; nie są po prostu pozostałymi krawędziami mesha.
7. Hard-surface zachowuje zamierzone ostre krawędzie. Suzanne nie narzuca stylu cube'owi.
8. Koszt całego widoku ma limit przed generowaniem dużych tablic CPU/GPU.
9. Panel i Rhai ustawiają te same typed kontrakty, z tą samą walidacją.
10. WGPU wykonuje packet; nie rozpoznaje uszu, nazw modeli ani intencji profilu.

### 1.2. Czego nie da się jednocześnie zagwarantować

Ślad przyklejony do powierzchni zmienia długość ekranową w skrócie perspektywicznym.
Ślad zawsze równomierny w pikselach musi czasem zmienić odwzorowanie na powierzchnię.
Nie można jednocześnie zachować idealnie stałej gęstości ekranowej, wszystkich
kotwic 3D, niezmiennej liczby kresek i niezmiennej parametryzacji w każdym ujęciu.

Proponowany kompromis: trwała rodzina ścieżek w 3D, szerokość w pikselach,
podzbiory LOD i przenoszona parametryzacja sylwetki. Konflikt stabilności 3D
z parametryzacją 2D jest opisany w
[Coherent Stylized Silhouettes](https://pixl.cs.princeton.edu/pubs/Kalnins_2003_CSS/kalnins2003css.pdf).
Nasz kontrakt i algorytm dopasowania poniżej są własnym projektem, nie kopią
implementacji autorów.

## 2. Wiedza rysunkowa przekładana na decyzje silnika

Rysownik interpretuje formę, zamiast zapisywać każdy podział techniczny modelu.
W projektowanym stylu ważne są: obrys, nakładanie brył, kierunek powierzchni,
zmiana waloru, rytm śladów i celowe uproszczenie. To założenia wybranego języka
rysunkowego, nie uniwersalne reguły dla wszystkich artystów.

| Informacja | Odpowiednik w obrazie | Konsekwencja implementacyjna |
| --- | --- | --- |
| Gdzie kończy się widoczna forma | Kontur i granice zasłaniania | Ekstrakcja zależna od kamery + widoczność |
| Gdzie powierzchnia naprawdę się załamuje | Wybrane creases | Jawne cechy powierzchni, nie każdy dihedral |
| Jak obraca się gładka forma | Linie wewnętrznej formy | Krzywizna, suggestive contours, apparent ridges |
| Jak powierzchnia przyjmuje światło | Walor i kreskowanie | Ciągłe pole tonu, aktywacja śladów |
| Jak poruszała się ręka | Przebieg, nacisk, odchylenie | Parametryzowany gest, nie jitter każdego wierzchołka |
| Co pozostawiło narzędzie | Ziarno i depozycja | Model materiału, oddzielny od geometrii |

**Mniej polygonów nie oznacza bardziej ludzkiego rysunku.** Można mieć bardzo
gęstą, dobrą powierzchnię obliczeniową i niewiele trafnie wybranych linii.
Upraszczamy opis rysunkowy, niekoniecznie obiekt używany do analizy i depth testu.
Rozróżnienie geometrii, wyboru linii, widoczności i stylizacji porządkuje
[Line Drawings from 3D Models](https://arxiv.org/abs/1810.01175).

### 2.1. Cztery niezależne źródła „wyglądu siatki”

- Kanciasty obrys: zbyt zgrubna geometria reprezentująca gładką powierzchnię.
- Wewnętrzna pajęczyna: krawędzie triangulacji błędnie uznane za ważne creases.
- Hatch łamany na ścianach: nieciągłe pole kierunków lub niepoprawny transport ścieżki.
- Regularna mozaika tonu: decyzje tonalne lub pokrycie zmieniają się per trójkąt.

Wobble nie usuwa żadnej z tych przyczyn. Rozmycie obrazu również nie rozstrzyga,
które linie powinny istnieć. Każdy przypadek wymaga osobnej diagnostyki.

## 3. Stan kodu i ryzyka do potwierdzenia testami

Poniższa tabela aktualizuje obserwacje dotyczące tego zakresu z wcześniejszego
audytu w `npr-drawing.md`. W worktree istnieją już m.in. powierzchniowe ścieżki,
LOD, historia krycia i indeksowany upload; nie należy implementować ich ponownie
pod nowymi nazwami.

| READ: plik i symbol | Zaobserwowane zachowanie | Wniosek |
| --- | --- | --- |
| `crates/engine/render-npr/src/feature.rs`, `classify_perspective_features` | Boundary/Silhouette/Crease wybierane z `TopologyEdge` na podstawie normalnych ścian | Kontur jest ograniczony do krawędzi polygonów |
| `crates/engine/render-npr/src/stroke.rs`, `chain_features` | Ścieżka przechowuje indeksy wierzchołków; ID wynika z krawędzi łańcucha | Zmiana łańcucha może zmienić gest |
| `crates/engine/render-npr/src/frame.rs`, `emit_surface_hatching_lane` | Rozstaw ziaren zależy od projekcji, a wyjściowe ID od końców ścieżki | Obrót i zoom mogą zmieniać zarówno ścieżkę, jak i jej tożsamość |
| Ten sam plik, `stable_surface_path_id` | Hash kwantowanych współrzędnych końców | Odwrócenie kolejności końców jest obsłużone; dowolny ruch już nie |
| `crates/engine/render-npr/src/field.rs`, `SurfaceDirectionField::build` | Pole z wygładzonych normalnych, osi rozpiętości i one-ring zmiany normalnej | To jeszcze nie estymator tensora krzywizny; oś obliczana dla obróconej geometrii może się zmieniać |
| `crates/engine/render-npr/src/hatching.rs`, `SurfaceHatchPath` | Osobne listy `points` i `faces` | Brak obowiązkowej, jednoznacznej kotwicy każdej próbki |
| Ten sam plik, `containing_selected_face` | Test współrzędnych barycentrycznych bez odległości od płaszczyzny | Punkt może pasować do rzutu na inną ścianę; szczególnie ryzykowne przy dwóch bliskich powierzchniach ucha |
| Ten sam plik, `trace_streamline_direction` | Krok po `face_tangent`, bez obowiązkowej rekonstrukcji punktu z kotwicy | Potrzebny test, że każda próbka naprawdę leży na aktualnej ścianie |
| `crates/engine/render-npr/src/temporal.rs`, `DrawingHistory::advance_packet` | Historia reguluje krycie, nie przenosi geometrii/parametryzacji; zegar i retencja aktualizują się per packet | Nie jest rozwiązaniem przerysowywania; potrzebny test niezależności od liczby obiektów |
| `plugins/gfx/npr-playground/src/render/mod.rs`, `rebuild_internal` | Transformacja pozycji przed budową packetu; seed zależy też od indeksu obiektu w mapie | Brak lokalnej przestrzeni powierzchni w wejściu algorytmu; dodanie wcześniejszego obiektu może zmienić seed innych |
| `crates/3d/mesh/src/geometry_asset.rs`, `load_gltf_geometry` | Weld pozycji per primitive, z kwantowaniem zależnym od rozmiaru | Nie zakładamy „braku weld”; potrzebujemy zachowania pochodzenia oraz rozdzielenia normalnych i topologii |

Walidacja odczytu: `rtk proxy rg -n "<symbol>" <dokładny-plik>`.
Odczyt nie upoważnia do zmiany wszystkich wykrytych miejsc naraz. Nie przypisujemy
zgłoszonego wyglądu ucha jednej udowodnionej przyczynie bez debug capture.

## 4. Jedna powierzchnia odniesienia, kilka reprezentacji danych

### 4.1. Źródło, powierzchnia rysunkowa i rasteryzacja

```text
mesh źródłowy + jawne intencje powierzchni
    -> przygotowana powierzchnia rysunkowa w przestrzeni lokalnej
         ├─ topologia, normalne, krzywizna i pole kierunków
         ├─ trwałe kotwice i bank ścieżek kreskowania
         └─ geometria do depth/fill oraz struktura zapytań widoczności
    -> analiza aktualnego widoku
    -> wybór linii i przeniesienie ich charakteru
    -> gest, clipping, tessellacja i packet
```

Nie należy wygładzać wyłącznie linii, a następnie ukrywać ich zupełnie innym,
zgrubnym modelem. Powierzchnia do analizy i zasłaniania musi opisywać tę samą
bryłę, z kontrolowanym błędem aproksymacji. Mesh źródłowy pozostaje nienaruszony.

### 4.2. Jawna intencja powierzchni

Planowane `SurfacePolicy` zawiera `Polygonal` albo `Smooth`, rewizję ustawień
przygotowania oraz maskę ostrych krawędzi/regionów. Mieszany obiekt ma regiony
gładkie rozdzielone cechami ostrymi. Nie jest rozpoznawany po nazwie `Suzanne`.

- `Polygonal`: geometryczne załamania mogą być intencją; aktualna ekstrakcja
  krawędzi pozostaje właściwym algorytmem dla tej deklaracji.
- `Smooth`: triangulacja jest próbkowaniem formy; jej zwykłe krawędzie nie są
  domyślnie crease. Jawne creases i rzeczywiste granice pozostają dostępne.
- Maska autorska ma pierwszeństwo. Opcjonalna propozycja ostrych krawędzi z kąta
  jest operacją przygotowania powierzchni z jawnym progiem, nie zgadywaniem WGPU.

### 4.3. Topologia nie jest tym samym co normalne i UV

Wierzchołki mogą być zdublowane z powodu UV, materiału lub rozdzielenia normalnych.
Potrzebujemy osobno: połączeń geometrycznych, narożników z normalnymi oraz regionów
rysunkowych. Nie sklejamy automatycznie obu stron cienkiego ucha tylko dlatego,
że są blisko. Tolerancja weld jest ograniczona, a operacja raportuje zmiany.

Przygotowanie sprawdza: skończoność danych, indeksy, pola trójkątów, orientację,
liczbę incydentnych ścian, boundary loops i self-intersection jako osobną klasę
ryzyka. Dane non-manifold otrzymują jawny błąd albo jawnie ograniczony tryb;
nie gubimy trzeciej ściany krawędzi w ciszy.

### 4.4. Wybór wygładzania

Rekomendowany pierwszy wariant `Smooth`: cache'owana, stała dla rewizji obiektu
subdivided proxy dla mesha trójkątnego. Punktem wyjścia jest schemat Loopa;
nie mylimy go z samym podziałem trójkąta na cztery bez zmiany pozycji.
Źródło: [Smooth Subdivision Surfaces Based on Triangles](https://www.microsoft.com/en-us/research/publication/smooth-subdivision-surfaces-based-on-triangles/).

Decyzje projektowe Amigo:

- Początkowo wybieramy stały poziom przygotowania, np. 1 albo 2, przez ustawienie
  i limit zasobów. Nie zmieniamy go co klatkę zależnie od kamery.
- Reguły boundary, crease i corner są osobnymi testowanymi maskami. Nie
  wygładzamy narożnika hard-surface razem z sąsiednią gładką łatą.
- Subdivision może zmienić objętość i grubość ucha. UI pokazuje obrys źródła oraz
  proxy w A/B; przekroczenie tolerancji kształtu blokuje zaakceptowanie ustawienia.
- Każdy poziom mnoży liczbę trójkątów około czterokrotnie. Limit jest sprawdzany
  przed alokacją. Brak zasobów nie oznacza cichego użycia innego stylu.
- Same interpolowane normalne poprawiają oświetlenie, ale nie wygładzają
  geometrycznego obrysu. To nie jest zamiennik tej operacji.

Docelowo ewaluator może obsłużyć rzeczywistą powierzchnię graniczną i adaptacyjną
tessellację. Ten etap wymaga trwałych adresów łat oraz kontroli błędu również dla
depth. Nie uznajemy go za bezkosztową zamianę pierwszego proxy.

Decymacja pozostaje narzędziem redukcji szumu wejściowego/kosztu z kontrolą błędu
kształtu, nie głównym sposobem stylizacji. Remesh nie przywróci brakujących fałd
ani wiedzy anatomicznej. „Przewidywana” linia musi wynikać z wybranej własności
formy lub jawnego akcentu artystycznego.

## 5. Geometria konturu: z krawędzi do krzywej

### 5.1. Definicja

Dla punktu `p` gładkiej powierzchni, normalnej jednostkowej `n(p)` i kamery `c`:

```text
v(p) = c - p
g(p) = dot(n(p), v(p))
generator konturu: g(p) = 0
kontur w obrazie: projekcja widocznej części generatora
```

To rozróżnia matematyczny generator od linii rzeczywiście widocznej. Dla polyhedra
generator wybiera krawędzie pomiędzy ścianą front-facing i back-facing; dla
powierzchni gładkiej szukamy zera na powierzchni, również wewnątrz trójkątów.
Definicja i porównanie: [DeCarlo i in., 2003](https://gfx.cs.princeton.edu/pubs/DeCarlo_2003_SCF/DeCarlo2003.pdf).

### 5.2. Proponowana implementacja etapowa

Pierwsze kryterium jakości dotyczy konturu gęstego proxy: jest zgodny z jego depth,
jest łączony i upraszczany jako cała linia. Następnie `Smooth` otrzymuje ekstrakcję
zer pola normalnych. To rozwój jednego trybu, nie druga ścieżka awaryjna.

Pseudokod zera na próbkowanym polu:

```text
for vertex i:
    g[i] = dot(analysis_normal[i], camera_local - position[i])

for undirected edge (i, j), exactly once:
    if signs differ:
        t = g[i] / (g[i] - g[j])
        crossing = SurfaceAnchor(edge_interpolation(i, j, t))
        cache crossing by canonical edge key

for triangle:
    connect its crossings using shared crossing handles
    preserve endpoints at boundaries, creases and singular events
```

To interpolacja zera **próbkowanego pola**, nie dokładne rozwiązanie równania
powierzchni granicznej. W perspektywie interpolacja normalnych i wektora widoku
nie czyni iloczynu automatycznie liniowym. Ewaluator dokładniejszej powierzchni
potrzebuje lokalnego rozwiązania zera i adaptacyjnego podziału także wewnątrz łat.

Zasady numeryczne:

- Jedna wartość przecięcia na współdzieloną krawędź; brak szpar przez dwa zaokrąglenia.
- Zera w wierzchołku i krawędzie niemal zerowe mają deterministyczną regułę
  własności oraz deduplikację. Nie pomijamy ich losowo epsilonem.
- Nie łączymy po samych współrzędnych 2D: przednia i tylna część ucha mogą się pokrywać.
- Błąd obrysu proxy i krzywej mierzymy w pikselach wobec gęstszej referencji.
- Gdy przybliżenie normalnych przesuwa generator względem depth, dopracowujemy
  powierzchnię/ekstrakcję. Zwiększanie depth bias nie jest rozwiązaniem geometrii.

### 5.3. Wygładzenie linii bez utraty znaczenia

Najpierw budujemy graf połączeń z informacją o powierzchni i rodzaju cechy.
Potem dzielimy go na gesty przy: autorskim narożniku, prawdziwej granicy, cusp,
rozgałęzieniu o niejednoznacznym kierunku lub dużej zmianie znaczenia linii.
Granica dowolnego trójkąta nie rozpoczyna nowego gestu.

Dopasowanie krzywej jest zadaniem z ograniczeniami. Przykładowa własna funkcja celu:

```text
E(curve) = sum_i weight_i * distance(curve(u_i), projected_sample_i)^2
         + lambda * integral |curve''(u)|^2 du
subject to:
    fixed semantic corners and endpoints
    maximum deviation <= simplify_error_px
    no bridge across visibility gaps or disconnected surface regions
```

Parametr i pochodzenie próbki pozostają zapisane również po uproszczeniu.
Nie wygładzamy depth przez dowolny spline ekranowy; pozycja/depth mają przypisanie
do pierwotnej ścieżki powierzchniowej. Na początku bezpieczniejsze są odcinki
adaptacyjnie próbkowane z ograniczonym odchyleniem niż swobodny spline przeskakujący
między fałdami. RDP może redukować próbki, ale sam nie tworzy gładkiej krzywej.

## 6. Linie wewnętrznej formy i skala analizy

### 6.1. Normalne i krzywizna

Normalna mówi, w którą stronę powierzchnia jest skierowana. Krzywizna opisuje,
jak szybko normalna zmienia się przy przemieszczaniu po powierzchni. Aktualny
wskaźnik `1 - dot(n_a, n_b)` nie ma jednostki odwrotności długości i zależy od
rozstawu próbek; nie zastępuje tensora krzywizny.

Przyjmujemy konwencję wypukłości dodatniej. Operator `S = Dn` w ortonormalnej
bazie stycznej daje dwie krzywizny główne `k1`, `k2` i kierunki `e1`, `e2`.
Estymację z różnic normalnych i agregacji na meshach opisuje
[Rusinkiewicz, 2004](https://gfx.cs.princeton.edu/pubs/Rusinkiewicz_2004_ECA/index.php).
Implementacja musi jawnie sprawdzić konwencję znaku na kuli, zamiast mieszać ją
z tekstami definiującymi operator kształtu jako `-Dn`.

Własny kontrakt analizy Amigo:

```text
CurvatureSample {
    principal_values, principal_directions,
    derivative_estimate,
    support_radius_object_units,
    confidence, residual, conditioning
}
```

Sąsiedztwo wybieramy według promienia/odległości po powierzchni, ograniczając
przejście przez ostre krawędzie. Sama liczba ringów nie oznacza tej samej skali
na dwóch nierówno zagęszczonych meshach. Macierze normalnych i tensorów trzeba
transportować do wspólnej bazy przed uśrednianiem.

Dla długości referencyjnej obiektu `L`: `k_hat = L*k` oraz
`derivative_hat = L^2 * D_unit_tangent(k)` pozwalają porównywać progi między
uniformowo skalowanymi obiektami. Skala ekranowa służy późniejszej selekcji detalu.
Krzywizna gładka w jednym sensie nie gwarantuje stabilnych trzecich pochodnych.

### 6.2. Suggestive contours

Nie są to dodatkowe creases ani wszystkie doliny. Wykorzystują radialną
krzywiznę w kierunku rzutu wektora do kamery na płaszczyznę styczną:

```text
w = v - n * dot(n, v)
r = w / |w|                        // tylko gdy |w| jest dostatecznie duże
k_r = dot(r, S * r)
kandydat: k_r = 0 oraz D_r(k_r) > 0
```

Do tego dochodzi widoczność i odrzucenie niestabilnych, słabych cech. To
charakterystyka z [Suggestive Contours for Conveying Shape](https://gfx.cs.princeton.edu/pubs/DeCarlo_2003_SCF/DeCarlo2003.pdf)
w przyjętej konwencji znaku. Nie wszystkie miejsca z dużą krzywizną spełniają
ten warunek; wypukła kula nie powinna dostać losowych linii wewnętrznych.

Projektowe filtry Amigo: confidence estymacji, dodatnia pochodna ponad progiem,
minimalna długość ekranowa, odległość od istniejącego konturu i stabilność na
dwóch pobliskich skalach analizy. Dopiero ich wynik trafia do budżetu linii.
Przy ujęciu prawie frontalnym `|w|` jest małe: pomijamy tę rodzinę w danym obszarze
z diagnostyką, nie dzielimy przez niemal zero. Histereza jest stosowana do oceny
ważności, nie do fałszowania widoczności. Literatura o zachowaniu w ruchu:
[Interactive Rendering of Suggestive Contours with Temporal Coherence](https://gfx.cs.princeton.edu/pubs/DeCarlo_2004_IRO/index.php).

### 6.3. Apparent ridges

Ta rodzina szuka maksimów zmiany normalnej względem obrazu, uwzględniając skrót
perspektywiczny. Dla lokalnego odwzorowania płaszczyzny stycznej na ekran `P`:

```text
Q = S * inverse(P)
q1 = largest_singular_value(Q)
ridge: local maximum of q1 along its principal screen direction
```

Nie wystarczy największa wartość własna `S`. Definicja i ograniczenia pochodzą
z [Apparent Ridges for Line Drawings](https://people.csail.mit.edu/tjudd/apparentLines.pdf).
Publikacja wyprowadza operator dla projekcji równoległej i używa lokalnego
przybliżenia dla perspektywy. Przy konturze odwzorowanie jest osobliwe.

W Amigo ta rodzina jest późniejszym etapem: osobny estymator confidence,
kontrola uwarunkowania projekcji, filtr maksimów, selekcja względem innych rodzin.
Nie rysujemy podwójnego obrysu tam, gdzie ta sama informacja jest już konturem.
Fałdy ucha są dobrym testem jakości, nie gwarancją, że ta metoda zawsze wybierze
anatomicznie najważniejszą linię.

### 6.4. Selekcja zamiast sumy wszystkich wykrytych linii

Własny model oceny kandydata może łączyć confidence, siłę cechy, długość w obrazie,
czytelność tonu i autorski priorytet. Składniki są normalizowane i diagnostyczne.
Nie ustalamy arbitralnego „jednego prawidłowego” zestawu wag.

Selekcja usuwa bliskie duplikaty tej samej informacji, krótkie odpryski oraz
szczegóły poniżej skali obrazu. Kontur i realne załamanie otrzymują pierwszeństwo
przed dodatkową linią formy i hatch. Odległość w obrazie nie może usunąć linii
na innej widocznej warstwie powierzchni tylko dlatego, że przecina jej projekcję.

## 7. Kreskowanie jako trwały układ ścieżek powierzchniowych

### 7.1. Kotwica i tożsamość

```rust
// Pseudokod docelowych kontraktów, typy nie istnieją jeszcze w tej postaci.
struct SurfaceAnchor {
    surface_revision: SurfaceRevision,
    primitive: PrimitiveId,
    triangle: TriangleId,
    barycentric: Vec3,
}

struct SurfacePathSample {
    anchor: SurfaceAnchor,
    rest_arc_length: f32,
}

struct SurfaceStroke {
    source_id: StrokeSourceId,
    family: HatchFamilyId,
    lod_rank: u64,
    samples: Vec<SurfacePathSample>,
}

struct StrokeInstanceKey {
    object: ObjectInstanceId,
    source: StrokeSourceId,
}
```

Na granicy trójkątów próbka ma kanonicznego właściciela; segment zachowuje też
trójkąt przejścia. `b0+b1+b2=1`, wartości są skończone i nieujemne z tolerancją.
Punkt jest rekonstruowany przez `p=b0*p0+b1*p1+b2*p2`. Nie wystarczy znalezienie
trójkąta, w którego rzucie mieści się dowolny punkt 3D.

Przy stałej rewizji powierzchni ID wynika z trwałego ziarna i rodziny, nie z:
końców ścieżki, współrzędnych świata, długości ekranowej, aktywnego indeksu obiektu,
numeru klatki ani liczby widocznych kresek. Hash jest skrótem pełnego klucza;
kolizje rozstrzygamy przez klucz, nie utożsamiamy przez sam kolor debug/u32.

Rewizja topologii unieważnia kotwice. Dowolny remesh nie ma automatycznej
korespondencji barycentrycznej; przeniesienie na nową powierzchnię jest osobną
operacją. Pierwszy etap nie obejmuje skinningu i deformacji ani nonuniform scale.
Obecne dodatnie uniform scale i rigid transforms wystarczą galerii. Rozszerzenie
transformacji wymaga poprawnego inverse-transpose normalnych i nowej analizy
krzywizny dla zmiany kształtu, nie tylko przemnożenia starego kierunku.

### 7.2. Bank ziaren niezależny od kamery

1. Dla ustalonej rewizji i layout seeda tworzymy kandydatów proporcjonalnie do
   pola powierzchni, nie jednakową liczbę na każdy trójkąt.
2. Kandydat ma stały porządek i priorytet. Rozmieszczenie typu blue-noise można
   uzyskać przez deterministyczną selekcję z nadmiarowego zbioru z ograniczonym
   testem odległości po powierzchni.
3. Rodziny gęstości są zagnieżdżone: rzadszy poziom to podzbiór gęstszego.
   Nie przeliczamy wszystkich ziaren dla nowego rozstawu w pikselach.
4. Ścieżki integrujemy i cache'ujemy w przestrzeni lokalnej. Maska bieżącego
   światła i widoku nie zmienia ich geometrycznych końców.
5. Światło, ton, LOD i widoczność wybierają fragmenty do pokazania. Każdy fragment
   zachowuje współrzędną i pochodzenie z pełnego śladu.

Inspiracją dla spójności między skalą a tonem są zagnieżdżone ślady w
[Real-time hatching](https://hhoppe.com/proj/hatching/).
Publikacja używa tonal art maps i teksturowania; nasz bank jawnych krzywych
powierzchniowych jest inną reprezentacją, wybraną dla sterowania pojedynczym gestem.

### 7.3. Pole kierunków bez zwrotu

Kreskowanie ma orientację osi, nie obowiązkowy zwrot strzałki: `d` i `-d` są
równoważne. Nie wolno ich naiwnie uśredniać do zera. Kierunki transportujemy
między bazami stycznymi i dopiero wyrównujemy znak albo reprezentujemy w 2D przez
`(cos(2*theta), sin(2*theta))`.

Pole opieramy na krzywiźnie i jawnych kierunkach artystycznych. Tam, gdzie
`|k1-k2|` jest małe, kierunek główny jest słabo określony. Na kuli i prawie
płaskiej łacie nie wymuszamy losowego „głównego kierunku” z błędu numerycznego.
Przyjmujemy jawny kierunek przewodni w przestrzeni obiektu, skracamy ślad albo
obniżamy jego ważność. To polityka domenowa z confidence, nie ukryte zgadywanie.

Wygładzanie pola używa odczytu poprzedniej iteracji i zapisu nowej, aby wynik
nie zależał przypadkowo od kolejności przeglądania krawędzi. Granice ostre są
barierami transportu. Osobliwości mają wykrywanie i limit kroków.

### 7.4. Integracja powierzchniowa

```text
trace(seed_anchor, field, rest_step, limits):
    state = seed_anchor
    repeat until length/step/visit budget reached:
        position = evaluate(state)
        direction = tangent_direction_at(state)
        direction = project_to_geometric_tangent(direction)
        align direction with transported previous direction

        advance through current triangle using barycentric derivatives
        if an edge is crossed:
            consume only the traveled part of the step
            select the adjacent smooth face from topology
            transport direction around the shared edge
            continue the remaining distance from a valid new anchor

        append anchor and accumulated rest arc length
        stop at a boundary, crease, singularity or recurrent short loop
```

Midpoint/RK2 może poprawić całkowanie pola, ale nie zwalnia ze śledzenia przejść
przez ściany. Odległość kroku zależy od krzywizny i tolerancji, jest wyrażona
w lokalnych jednostkach. Przejście przez wierzchołek ma deterministyczną regułę
i limit liczby przejść; nie może tworzyć nieskończonej pętli epsilonowych skoków.
Nie przeszukujemy całego mesha, aby po każdym kroku odzyskać zgubioną ścianę.

## 8. Humanizacja, która nie resetuje się podczas ruchu

### 8.1. Trzy przestrzenie i dwa niezależne seedy

- Przestrzeń powierzchni: położenie i rest arc length kreski.
- Przestrzeń gestu: nacisk, odchylenie, faza nieregularności, wariant poprawki.
- Przestrzeń papieru/obrazu: mikrostruktura podłoża i szerokość ekranowa.

Oddzielamy `layout_seed` od `gesture_seed`; papier ma własny seed materiału.
Globalny seed może deterministycznie wyprowadzać je wszystkie, ale polecenie
„nowy wariant gestu” nie przebudowuje powierzchni ani rozmieszczenia hatchy.

### 8.2. Parametr śladu

Podział, clipping, LOD i chwilowe zasłonięcie nie zerują parametru. Zapisujemy
`u` jako pochodzenie w ścieżce, a nie każdorazowo `screen_length / visible_length`.
Zmiana długości widocznego końca nie może przesunąć ziarna na drugim końcu ucha.

Własny model humanizacji:

```text
offset(u) = long_bend(u, gesture_seed, stroke_id)
          + a_mid * correlated_noise(u / lambda_mid, seed_mid)
          + a_fine * band_limited_noise(u / lambda_fine, seed_fine)

pressure(u) = clamp(envelope(u) + slow_pressure_variation(u), 0, 1)
width_px(u) = tool_response(pressure(u), nib_angle(u), role)
```

To model proceduralny, nie udowodniony model biomechaniki ręki. Korelacja daje
długie odchylenia i rytm; niezależny szum na każdym wierzchołku daje drut/piłę.
Odchylenie jest ograniczone tolerancją czytelności formy, szczególnie w wąskich
szczelinach. Wzrost amplitudy nie może omijać zasad visibility.

Końce gestu mają własny profil nacisku. Koniec powstały wskutek zasłonięcia nie
jest końcem pociągnięcia ręki i nie dostaje automatycznie taperu. Poprawka jest
osobnym, skorelowanym gestem tej samej intencji, a nie identyczną kopią każdej linii.

### 8.3. Ołówek i tusz korzystają z tej samej ścieżki

Ołówek może zmieniać pokrycie i ziarno wraz z naciskiem. Stalówka może mieć
szerokość zależną od kąta kierunku ruchu względem osi końcówki. Nie mieszamy
rozmieszczenia linii z reakcją narzędzia. Zachowujemy istniejący rozwój `tool.rs`
i materiału zamiast dopisywać własne reguły w WGPU.

Papier nieruchomy w obrazie i kreska poruszająca się po obrazie naturalnie dają
zmianę kontaktu z ziarnem. To nie jest zmiana tożsamości kreski. Testy stabilności
najpierw wyłączają grain papieru, potem oceniają go oddzielnie. „Stabilny” nie
oznacza identycznego koloru każdego poruszającego się piksela.

## 9. Śledzenie sylwetki: stan geometryczny i stan gestu

### 9.1. Dlaczego kotwica hatcha nie wystarcza

Sylwetka nie jest trwałą rysą na modelu: przy ruchu zmienia się zbiór punktów
spełniających `n dot v = 0`. Przechowywanie tylko starej krzywej 3D spowodowałoby,
że stara sylwetka zostałaby wewnątrz nowej bryły. Potrzebna jest bieżąca ekstrakcja
i przenoszenie charakteru śladu do odpowiadających fragmentów.

Rozdzielamy więc `FeatureCurve` — aktualną geometrię cechy — od
`TrackedStrokeSpan` — fragmentu z tożsamością i przenoszonym parametrem gestu.
Inspiracja: przenoszenie próbek i parametryzacji w
[Coherent Stylized Silhouettes](https://pixl.cs.princeton.edu/pubs/Kalnins_2003_CSS/kalnins2003css.pdf).

### 9.2. Proponowane dopasowanie Amigo

```text
begin_view_frame(view_id, frame_id, simulation_time)
extract current feature curves
reproject previous surface samples with current object transform and camera
build bounded screen-space index of current curve segments

for previous sample:
    candidates = nearby segments of same object and compatible feature family
    reject disconnected sheets / implausible surface displacement
    reject tangent reversal unless explained by canonical orientation
    reject depth/visibility ownership conflict
    evaluate remaining matches and transfer old gesture coordinate

group matches into monotone parameter intervals
resolve splits and merges using interval provenance
initialize genuinely new spans deterministically
end_view_frame_once()
```

Przykładowy koszt projektowy:

```text
cost = wd * (screen_distance / allowed_distance_px)^2
     + wt * (1 - abs(dot(old_tangent, new_tangent)))
     + ws * (surface_distance / allowed_surface_distance)^2
     + wp * parameter_order_penalty
```

Limity odległości uwzględniają ruch i skalę obiektu, nie rosną bez ograniczeń.
Odległość powierzchniową przybliża ograniczone przejście po sąsiedztwie; samo
euclidean distance jest niewystarczające między obiema stronami małżowiny.
Remisy rozstrzyga kanoniczny klucz. Dopasowania mają minimalny confidence.
Odrzucenie niezgodnego dopasowania jest lepsze niż przeniesienie gestu na inne ucho.

### 9.3. Split, merge i ponowne odsłonięcie

- Split: fragmenty dziedziczą przedziały parametru i pochodzenie rodzica.
- Merge: zachowujemy lokalne przedziały obu gestów. Nie rozciągamy ID zwycięzcy
  automatycznie na całą nową linię; możemy zostawić naturalną przerwę między gestami.
- Powrót po zasłonięciu: hatch wraca z banku ścieżek; kontur próbuje dopasowania
  tylko w ograniczonej retencji i przy wiarygodnej korespondencji.
- Skok kamery: jawny reset śledzenia konturów, bez przebudowy lokalnego banku hatchy.
- Topology change: reset powierzchni i wszystkich zależnych kotwic.

Kontrolowana utrata korespondencji jest nieunikniona przy narodzinach/zanikach
konturu i skokach widoku. Kryterium odbioru obejmuje brak globalnego przetasowania
śladów, nie niemożliwą obietnicę wiecznej tożsamości każdej sylwetki.

### 9.4. Jeden zegar na widok, a nie na obiekt

Historia jest kluczowana `(ViewId, ObjectInstanceId, feature/span)`. Retencja
starzeje się raz na logiczną klatkę widoku. Wywołanie extractora drugi raz dla
tego samego `frame_id` nie postępuje czasu. Renderowanie sześciu obiektów nie
przyspiesza zanikania historii sześciokrotnie.

Bank powierzchni może być współdzielony między widokami. Śledzenie sylwetki
nie może: panel edytora i główna kamera mają różne generatory konturu.
Nie zapisujemy mutexów, cache ani historii w presetach sceny.

## 10. Dokładna semantyka „wyłącz animację kresek”

```rust
enum StrokeMotionMode {
    Stable,
    RedrawOnMotion,
}

struct NprMotionPolicy {
    mode: StrokeMotionMode,
    redraw_hz: f32,
    redraw_strength: f32,
    appearance_fade_seconds: f32,
}
```

Domyślnie `Stable`. UI może przedstawić enum jako przełącznik
„Przerysowuj kreski podczas ruchu”, ale API nie ma niejednoznacznego
`temporal_enabled`. **Stabilizacja działa także wtedy, gdy przerysowywanie jest
wyłączone.** Nie wyłączamy wówczas trackera potrzebnego do stabilności sylwetki.

| Zdarzenie | Stable | RedrawOnMotion |
| --- | --- | --- |
| Obrót/pan/orbit | Aktualna projekcja i visibility; zachowany wariant gestu | To samo + kontrolowane zmiany wariantu |
| Zoom | Reprojekcja i dobór podzbioru LOD | To samo + warianty, jeśli zoom przekracza próg ruchu |
| Brak ruchu | Brak nowych wariantów | Brak nowych wariantów |
| Nowa widoczna cecha | Nowy span, opcjonalne wejście krycia | Takie samo zarządzanie cechą |
| Fade ustawione na 0 | Natychmiastowe krycie aktualnych linii | Natychmiastowe krycie aktualnych linii |
| Wyłączenie przerysowywania | Zachowanie obecnego wariantu; dalsza geometria normalnie | Przejście do Stable, bez wymuszonego reroll |

Przerysowywanie zmienia gest, nie bank ziaren, fizyczny kształt obiektu ani
geometryczną definicję konturu. Ruch mierzymy z odpowiednio rozłożonych kotwic
przereprojektowanych do nowego widoku, nie tylko z centrum bounding boxa.
Obrót symetrycznego obiektu również może poruszać jego kreskowaniem.

Zegar wariantów jest jawny. Docelowo wykorzystuje stałe kroki czasu symulacji
i znormalizowany sygnał ruchu, z progiem wejścia/wyjścia. FPS renderera ani tempo
odświeżania panelu nie są częstotliwością zmiany wariantu. W testach z różnym FPS
odtwarzamy te same próbki symulacji; zmiana sposobu próbkowania inputu ma osobną
tolerancję i nie może być błędnie nazywana gwarancją bitowej identyczności.

Przykład intencji, nie gotowego scheduler API:

```text
if mode == RedrawOnMotion and motion_gate_active:
    redraw_clock += fixed_simulation_dt
variant_epoch = floor(redraw_clock * redraw_hz)
gesture = seeded_gesture(stroke_key, base_gesture_seed, variant_epoch)
```

Tryb stabilny zachowuje ostatni wybrany wariant. Polecenie „Nowy wariant gestu”
jest osobnym, jednorazowym zdarzeniem. Fade to jeszcze inny mechanizm:

```text
if tau == 0: weight = target
else: weight += (target - weight) * (1 - exp(-dt / tau))
```

Fade nie ma prawa pozostawić śladu ponad zasłaniającym obiektem. Aktualna
widoczność obowiązuje natychmiast; retencja zapisuje tożsamość, nie duchy starych
pozycji ekranowych.

## 11. Zoom i LOD bez „gotowania” obrazu

Obecnego `plugins/gfx/npr-playground/src/zoom.rs` nie zastępujemy. Wygładzanie
ruchu kamery i stabilność rysunku to różne warstwy.

Proponowany LOD operuje na stałym rankingu banku śladów. Oddalenie wyłącza
mniej ważne ślady; przybliżenie odzyskuje te same ślady z tym samym `u` i seedem.
Histereza zapobiega przełączaniu dokładnie na granicy progu. Fade LOD jest
opcjonalny; jego wyłączenie nie może przebudować ścieżek.

Zagęszczenie w obrazie może być niejednorodne. Używamy lokalnej skali projekcji
i ograniczonej konkurencji pobliskich śladów, zamiast jednego wskaźnika rozmiaru
całego obiektu. Kanoniczny rank i priorytet rodzin ograniczają przetasowania.
Szerokość wyrażamy w **fizycznych pikselach render targetu**; UI opisuje to
jawnie i poprawnie przelicza skalę DPI z punktów egui.

Po wyczerpaniu najgęstszego banku pokazujemy diagnostykę jakości. Zmiana poziomu
przygotowania jest świadomą operacją, nie przypadkowym nagłym reseed przy scrollu.

## 12. Widoczność, głębokość i ograniczenia stylizacji

Depth, fill i stroke zachowują globalną kolejność dla wszystkich obiektów
wewnątrz istniejącego węzła World. Pencil może nie mieć fill koloru, ale nadal
musi mieć geometrię zasłaniającą. Nie usuwamy już istniejącego kontraktu occluderów.

Planowanie korzysta z CPU zapytań powierzchniowych/BVH do dzielenia ścieżek
i unikania kosztu ewidentnie ukrytych fragmentów. Końcowy GPU depth test jest
nadal autorytatywny dla fragmentów. Pierwsza implementacja nie wymaga synchronicznego
odczytu depth z GPU do CPU co klatkę.

Inwarianty:

- Widoczność uwzględnia wszystkie obiekty NPR w widoku, nie tylko self-occlusion.
- Geometryczny clipping następuje przed dzieleniem przez `w`; nowe próbki
  zachowują parametr pochodzenia, aby near plane nie resetował gestu.
- Znormalizowane depth i inne atrybuty zachowują poprawne zasady interpolacji.
  Materiał korzystający z interpolowanego parametru powierzchni potrzebuje
  odpowiedniej korekcji perspektywy; nie interpolujemy liniowo eye-depth w 2D.
- Tessellacja zachowuje semantykę końca gestu versus końca clippingu/visibility.
- Nudge jest ograniczony i deklarowany; nie służy do naprawiania niezgodnych brył.
- Zewnętrzne odchylenie sylwetki zachowuje depth źródła oraz test względem
  bliższych powierzchni. Duże odchylenia w wąskiej szczelinie są redukowane.

Dla małżowiny testujemy obie bliskie ścianki, widok styczny i obiekt zasłaniający
ucho. Zwykły test na izolowanym sześcianie nie udowadnia poprawności tego etapu.

## 13. Kontrakty wykonania i inwalidacja

```rust
struct NprPreparedSurface {
    revision: SurfaceRevision,
    geometry: SurfaceGeometry,
    adjacency: SurfaceAdjacency,
    analysis: SurfaceAnalysis,
    visibility_acceleration: SurfaceQueries,
}

struct NprDrawingInstance {
    id: ObjectInstanceId,
    surface: SurfaceHandle,
    transform: RigidUniformTransform,
    profile: DrawingProfileHandle,
}

struct DrawingFrameContext {
    view: ViewId,
    frame: FrameId,
    simulation_time: Seconds,
    camera: PerspectiveCamera,
    viewport_physical_px: [u32; 2],
}

// Domena posiada algorytmy; plugin posiada instancję sesji i jej cykl życia.
fn advance_drawing(
    session: &mut DrawingHistory,
    surfaces: &PreparedSurfaceStore,
    instances: &[NprDrawingInstance],
    context: DrawingFrameContext,
    policy: NprMotionPolicy,
    budget: DrawingBudget,
) -> Result<NprFrameOutput, DrawingError>;
```

Nazwy spinają odpowiedzialności; nie wymagają od razu rejestru wszystkich
możliwych rendererów. Kontrakt zachowuje neutralność względem WGPU. `frame.rs`
orkiestruje etapy zamiast zawierać ich wszystkie pętle. `DrawingHistory` rozwijamy
w miejscu; nie powstaje równoległy `TemporalV2`.

| Zmiana | Co aktualizujemy | Co zachowujemy |
| --- | --- | --- |
| Transformacja rigid/uniform | Projekcja, ton, visibility, kontury | Lokalna powierzchnia, kotwice, bank hatchy |
| Kamera/FOV | Analiza widoku, tracking, LOD, tessellacja | Bank powierzchniowy i layout seed |
| Resize/DPI | Projekcja, pikselowe kryteria, target depth | ID, powierzchnia, pipeline tego samego formatu |
| Światło/kontrast | Pole tonu i aktywność fragmentów | Pełne ścieżki i ich parametr |
| Szerokość/nacisk/grain | Gest/materiał/tessellacja według zależności | Położenia ziaren, topologia |
| Kierunek kreskowania | Pole/ścieżki zależne od zmienionej polityki | Geometria proxy, niezmienione cechy |
| Layout seed | Bank ziaren i ścieżek | Powierzchnia i jej analiza |
| Gest seed/nowy wariant | Parametry gestu | Bank ścieżek i source ID |
| Debug view | Kolor/diagnostyczny packet | Semantyczne cache i historia gestu |
| Topologia/proxy/crease mask | Powierzchnia i zależne banki/historie | Niezwiązane obiekty |
| Camera cut | Korespondencja konturów danego widoku | Bank powierzchni i inne widoki |
| Format/device | Właściwe zasoby WGPU | Dane domenowe, o ile nadal aktualne |

Nie haszujemy całego `Debug` profilu jako jednej rewizji wszystkich cache.
Jawne klucze zależności zapobiegają resetowi ziaren przy zmianie koloru panelu
czy drobnego parametru papieru. Przebudowa drogiego zasobu jest atomowa; do czasu
publikacji nowej rewizji stara pozostaje spójnym snapshotem z oznaczeniem oczekiwania.

## 14. Cała klatka: pseudokod implementacyjny

```text
extract immutable scene/control snapshot
ensure prepared surfaces and persistent hatch banks for requested revisions
begin history for (view_id, frame_id) once

for each instance ordered by stable instance id:
    update object-to-view mapping
    evaluate continuous tone on the authoritative surface
    extract current boundary/crease/contour and enabled form-line candidates
    track view-dependent feature spans against previous frame
    reproject persistent surface paths
    compute visibility intervals and importance without resetting path coordinates
    append candidates with stable keys to the view-wide candidate set

select candidates under global planning and memory budgets
for each selected span:
    choose stable or motion-driven gesture variant
    apply pressure/offset/material with preserved parameterization
    clip with provenance; adaptively tessellate under reserved capacity

finish history aging once
emit occluders + fills + strokes + bounded diagnostics
bridge output into existing WGPU frame packet
execute global depth / fill / stroke passes
```

Referencyjny render bez historii jest osobnym jawnym trybem tej samej domeny:
ustalone wejście, seed, wariant i LOD dają deterministyczny packet. Interaktywny
tracking jest zależny od historii. Dwa dojścia do tego samego ujęcia mogą dać
inne poprawne przypisanie gestów sylwetki. Golden sekwencji zapisuje więc także
początek i przebieg symulacji; nie porównuje interaktywnej historii z bezstanowym
zrzutem tak, jakby miały gwarantować bitowo ten sam obraz.

## 15. Panel, metadane i Rhai

### 15.1. Grupowanie UX

- **Powierzchnia:** Polygonal/Smooth, jakość proxy, zachowanie ostrych krawędzi,
  podgląd źródła i proxy; ustawienia per obiekt/region.
- **Linie formy:** contour/crease/suggestive/apparent, skala analizy, minimalna
  ważność i długość, uproszczenie w pikselach.
- **Kreskowanie:** pole kierunków, rozstaw, liczba rodzin, zakres długości,
  confidence; diagnostyka LOD i zajętości banku.
- **Gest i narzędzie:** pewność, amplituda, nacisk, końce, poprawki; istniejące
  profile narzędzi, bez nowego zestawu równoległych presetów.
- **Ruch rysunku:** „Przerysowuj podczas ruchu” OFF domyślnie, częstotliwość
  i siła aktywne tylko przy ON, niezależne wejście krycia, „Nowy wariant gestu”.
- **Diagnostyka:** surowe krawędzie, normalne, curvature confidence, kotwice,
  ID źródłowe i śledzone, widoczność, zmiany ID i budżet.

Kontrolki obrotu, pauzy i kamery pozostają oddzielnie. Tooltip trybu stabilnego:
„Zachowuje charakter kresek. Kontury i ukrywanie linii nadal reagują na widok”.
Droga operacja pokazuje stan przygotowania; suwaki nie publikują niespójnych
połówek ustawień. Undo i preset sceny odtwarzają politykę, ale nie losowe mutexy
czy bieżące bufory historii.

### 15.2. Planowane ścieżki

Prefiks istniejący: `world.npr.settings.NprSettings.`. Sufiksy niżej są nowe:

```text
motion.mode                      // enum: Stable | RedrawOnMotion
motion.redraw_hz                 // częstotliwość wariantu, nie FPS aplikacji
motion.redraw_strength
motion.appearance_fade_seconds   // 0 wyłącza przejście krycia
reroll_gesture                   // zdarzenie/polecenie, nie utrzymywany bool
object.surface.mode              // alias do wybranego obiektu
object.surface.subdivision_level
object.surface.analysis_radius
```

```rhai
// Pseudokod bindingów po dodaniu ich do istniejącego providera.
let p = "world.npr.settings.NprSettings.";
world.controls.set(p + "motion.mode", "Stable");
world.controls.set(p + "motion.appearance_fade_seconds", 0.0);
```

Metadane definiują typ, zakres, jednostki, opis, możliwość zapisu i koszt
inwalidacji. Provider jest jedynym właścicielem walidacji; UI i Rhai jej nie
duplikują. Typed Rust definiuje algorytm i profile. YAML przechowuje layout
i wartości, nie kod ekstrakcji powierzchni.

Pełny preset sceny obejmuje motion i surface. Preset samego wyglądu nie zmienia
kamery, pauzy ani polityki ruchu. Zmiana schematu aktualizuje provider, authored
presety i testy w jednej operacji. Nieobsługiwana wersja pliku otrzymuje jawny
błąd, bez cichej reinterpretacji i automatycznego nadpisania.

Kompatybilność z przyszłym edytorem wynika ze wspólnego metadata/control contract,
nie z dostępu edytora do `NprPlaygroundState`. Nie zmieniamy implementacji ogólnego
panelu egui ani hosta aplikacji tylko po to, by dodać kontrolkę NPR.

## 16. Budżety i diagnostyka

Zgłoszony crash z buforem 460 259 520 B przy limicie device 268 435 456 B jest
przypadkiem regresyjnym, również dla nowego pipeline'u. Dzielenie uploadu chroni
pojedynczy bufor; nie gwarantuje rozsądnego całkowitego kosztu pamięci lub klatki.

Oddzielne limity: przygotowana powierzchnia, kandydaci ziaren, kroki integracji,
próbki ścieżek, porównania trackera, aktywne stroke spans, tessellacja, upload
oraz rezydencja wszystkich ramek GPU w locie. Kandydaci odrzuceni z budżetu
nie mogą wcześniej alokować całej gotowej tessellacji.

Proponowane początkowe założenia do pomiaru, **nie potwierdzone osiągi**:

- 512×512: referencje geometryczne i obrazy.
- 1920×1080, sześć modeli, RTX 3070 Ti: cel 60 FPS po przygotowaniu cache;
  raportujemy także CPU, adapter, tryb kompilacji, medianę i p95, nie sam GPU.
- Globalny limit payloadu stroke CPU/upload na widok startowo 64 MiB; dokładny
  limit geometrii wyliczany przez `size_of`, checked arithmetic i limit indeksów.
- Pojedynczy upload nie większy niż istniejący limit chunków ani device limit.
  Całkowity koszt obejmuje staging i liczbę klatek w locie.
- Twarde rezerwy na kontur i ważne creases; hatch ustępuje najpierw. Rezerwy
  również mają limit, więc nie ma rodziny, która może alokować nieskończenie.

Przykładowe liczniki `npr.*`:

```text
surface.source_triangles / proxy_triangles / invalid_regions
surface.anchor_residual_max / analysis_confidence_low
paths.cached / active / retraced / max_integration_steps
features.raw / selected / duplicate_rejected / weak_rejected
tracking.matched / born / split / merged / rejected / reset_reason
tracking.unexpected_id_churn / reprojection_residual_px
motion.mode / variant_changes / fade_seconds
lod.bank_level / changes / exhausted
memory.prepared_bytes / stroke_bytes / upload_bytes / resident_gpu_bytes
budget.rejected_candidates / rejected_samples
timing.prepare_ms / extract_ms / track_ms / tessellate_ms
```

`unexpected_id_churn` liczy utratę tożsamości tylko na fragmentach, dla których
istnieje wiarygodna korespondencja. Narodziny konturu nie są automatycznie błędem.
Statystyki nie wymagają logowania pełnego raportu co klatkę w release.

## 17. Mapa operacji implementacyjnych

Wszystkie operacje niżej są **przyszłą pracą**, nie zmianami wykonanymi przez
napisanie tej specyfikacji. Zachowujemy lokalne zmiany użytkownika. Pełne nazwy
plików i proponowane symbole są punktem startu, a zakres każdej operacji należy
ponownie potwierdzić na aktualnym worktree.

Walidacje skrócone w tabelach:

- `N`: `rtk cargo check -p amigo-render-npr`, potem wskazany test domenowy.
- `P`: po zielonym N, `rtk cargo check -p amigo-npr-playground-plugin`, potem
  `rtk cargo test -p amigo-npr-playground-plugin --test control_tests`.
- `G`: po zielonym N, `rtk cargo check -p amigo-render-api`, następnie
  `rtk cargo check -p amigo-render-wgpu` i wskazany test WGPU.
- `D`: `rtk git diff --check` dla zmienionej dokumentacji.

### 17.1. Fundamenty i geometria

| Operacja i dokładny plik | Symbol / intencja | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `crates/3d/mesh/src/geometry_asset.rs` | `load_gltf_geometry`: zachowanie primitive/corner provenance i jawnych cech potrzebnych analizie | `rtk cargo check -p amigo-3d-mesh`; test importu szwu i dwóch bliskich ścian | Nie dodawać stylowania NPR do importera |
| ADD `crates/engine/render-npr/src/surface.rs` | `SurfacePolicy`, `NprPreparedSurface`, `SurfaceAnchor`, walidacja rewizji i kotwic | N; `surface_tests` | Nie modyfikować plików modelu źródłowego |
| MODIFY `crates/engine/render-npr/src/topology.rs` | `build_topology`: potrzebne sąsiedztwo i jawne bariery, zachowując walidację | N; `surface_tests` i istniejące testy topologii | Nie tracić wykrywania non-manifold |
| ADD `crates/engine/render-npr/src/subdivision.rs` | Przygotowanie `Smooth` proxy, maski boundary/crease/corner, trwałe adresy poziomu | N; `surface_tests` | Nie zmieniać Polygonal ani poziomu proxy co frame |
| ADD `crates/engine/render-npr/src/curvature.rs` | Tensor, kierunki, confidence, pochodne i skala wsparcia | N; `form_line_tests` | Nie nazywać dot-normal-turn dokładną krzywizną |
| MODIFY `crates/engine/render-npr/src/feature.rs` | `classify_perspective_features` i `FeatureSegment`: wspólny kontrakt cechy z kotwicami | N; `form_line_tests` | Nie usuwać intencji Boundary/Crease |
| ADD `crates/engine/render-npr/src/contour.rs` | Generator gładkiego konturu, wspólne przecięcia, deduplikacja osobliwości | N; `form_line_tests` | Nie utożsamiać generatora z widocznością |
| ADD `crates/engine/render-npr/src/form_lines.rs` | Suggestive contours i później apparent ridges jako jawne rodziny | N; `form_line_tests` | Nie rysować sumy wszystkich kandydatów bez selekcji |
| MODIFY `crates/engine/render-npr/src/stroke.rs` | `chain_features`: graf i ścieżki kotwic zamiast wyłącznie vertex IDs | N; `form_line_tests` | Nie łączyć po samym ekranowym sąsiedztwie |

### 17.2. Ścieżki, gest i ruch

| Operacja i dokładny plik | Symbol / intencja | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `crates/engine/render-npr/src/field.rs` | `SurfaceDirectionField::build`: lokalne pole, transport, confidence, odczyt poprzedniej iteracji | N; `surface_path_tests` | Nie narzucać osi świata wszystkim modelom |
| MODIFY `crates/engine/render-npr/src/hatching.rs` | `SurfaceHatchPath`, `trace_surface_streamline`: próbki z kotwicami, bank śladów, bounded integration | N; `surface_path_tests` | Nie dobierać start face przez projekcję na dowolną ścianę |
| MODIFY `crates/engine/render-npr/src/lod.rs` | `HatchLodState`: selekcja trwałego podzbioru i histereza | N; `motion_tests` | Nie regenerować ziaren przy zoomie |
| MODIFY `crates/engine/render-npr/src/temporal.rs` | `DrawingHistory`, `NprMotionPolicy`: historia widoku, zegar raz na frame, tracking spans, niezależny fade | N; `motion_tests` | Nie rysować starych pozycji jako duchów |
| MODIFY `crates/engine/render-npr/src/gesture.rs` | `sample`: trwały parametr, osobny wariant i seed gestu | N; `motion_tests` | Nie używać numeru renderowanej klatki jako RNG |
| MODIFY `crates/engine/render-npr/src/tessellation.rs` | `tessellate_polyline_variant`: źródłowy parametr, semantyka końców, ograniczony błąd i alokacja | N; `surface_path_tests` | Nie resetować taperu na granicy trójkąta/occlusion |
| MODIFY `crates/engine/render-npr/src/frame.rs` | `build_packet_with_topology`, `NprRenderStats`: orkiestracja przygotowanej powierzchni, instancji i liczników etapów | N; wszystkie cztery nowe suites | Nie zostawiać równoległej implementacji tego samego trybu |
| DELETE `crates/engine/render-npr/src/frame.rs` | `stable_surface_path_id` po przełączeniu wszystkich jego użytkowników na source keys | N; `motion_tests`; targeted `rg` live/test refs | Nie usuwać przed zakończeniem migracji |
| MODIFY `crates/engine/render-npr/src/budget.rs` | `select_ranked`: wspólna polityka widoku i koszt przed tessellacją | N; `motion_tests` | Nie mnożyć globalnego limitu przez liczbę obiektów |
| MODIFY `crates/engine/render-npr/src/debug.rs` | `NprDebugView`: jawne podglądy powierzchni, kotwic, confidence i korespondencji; dane tworzy domena | N; `motion_tests` i `form_line_tests` | Nie wywodzić tożsamości z koloru debug |
| MODIFY `crates/engine/render-npr/src/lib.rs` | Eksporty nowych kontraktów i usunięcie zastąpionych eksportów | N | Bez nazw `v2` i shimów |

### 17.3. Integracja i panele

| Operacja i dokładny plik | Symbol / intencja | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `plugins/gfx/npr-playground/src/state.rs` | `Settings`, `ObjectSettings`, provider: motion/surface, zakresy, undo i atomowe snapshots | P | Algorytmy powierzchni nie należą do pluginu |
| MODIFY `plugins/gfx/npr-playground/src/render/mod.rs` | `NprPlaygroundRenderService`, `stats`: przechowywanie zasobów domeny, stabilne instance IDs, view/frame context i agregacja liczników | P; waterfall tests | Bez kopiowania ekstrakcji i integratora do serwisu |
| MODIFY `plugins/gfx/npr-playground/src/plugin.rs` | Update/RenderExtract: jawny czas, reset sesji i publikacja diagnostyk przez istniejący mechanizm | P; waterfall tests i diagnostics tests | Nie zmieniać zegara obrotu przez redraw_hz |
| MODIFY `mods/npr-playground/ui/npr.panel.yml` | Nowe sekcje/wiązania, domyślnie Stable, opis fade, readiness | P; walidacja `PanelDocument` | Bez logiki rendererowej w YAML |
| MODIFY `mods/npr-playground/scenes/gallery/scene.rhai` | Zdarzenie nowego wariantu i ewentualne akcje debug przez controls | P; testy sceny | Bez prywatnego dostępu do serwisu renderującego |
| MODIFY `mods/npr-playground/scenes/cube/scene.rhai` | Te same dostępne akcje wspólnego panelu | P; testy sceny | Zachować Polygonal dla autorskiego cube |
| READ `crates/engine/render-api/src/npr.rs` | `NprDrawCommand`: ocena, czy gotowy packet nadal wystarcza | `rtk cargo check -p amigo-render-api` tylko jeśli kontrakt zmieniony | Bez dodawania historii domenowej do backendu |
| READ `crates/runtime/bundles/src/render_extractor_bridges/world_3d.rs` | Bridge NPR: powinien nadal kopiować wynik | Check owner bundle dopiero jeśli plik wymaga modyfikacji | Bez domenowych heurystyk |
| MODIFY `crates/engine/render-wgpu/src/renderer/npr.rs` | Tylko wykonanie nowych atrybutów materiału/debug, gdy packet tego wymaga | G; testy shaderów i limitów | Bez wyboru krzywizny, śladów, seeda |
| MODIFY `crates/engine/render-wgpu/src/renderer/service/render/world.rs` | `render_npr_commands`: zgodne occludery i ograniczony upload nowych packetów | G; offscreen test | Zachować globalne passy i zwykły MeshDrawCommand |

Nie ma obecnie uzasadnienia do zmiany hosta w `crates/apps/app/src/main.rs`
ani przebudowy FrameGraph. App pojawia się niżej wyłącznie jako właściciel
istniejących testów integracyjnych offscreen.

### 17.4. Testy i dokumentacja to część operacji

| Operacja i dokładny plik | Intencja | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| ADD `crates/engine/render-npr/tests/support/mod.rs` | Wspólne analityczne powierzchnie i thin-shell fixtures dla nowych suites | Nowe owner suites poniżej | Bez zależności testów matematycznych od zewnętrznych assetów |
| ADD `crates/engine/render-npr/tests/surface_tests.rs` | Proxy, kotwice, szwy, thin-shell, zachowanie creases | `rtk cargo test -p amigo-render-npr --test surface_tests` | Nie opierać testów wyłącznie na Suzanne |
| ADD `crates/engine/render-npr/tests/surface_path_tests.rs` | Transport, integracja, ciągłość, parametryzacja | `rtk cargo test -p amigo-render-npr --test surface_path_tests` | Bez testów tylko liczby wierzchołków |
| ADD `crates/engine/render-npr/tests/form_line_tests.rs` | Krzywizna, zero crossings, selekcja i analiza skali | `rtk cargo test -p amigo-render-npr --test form_line_tests` | Nie żądać identycznych ID po dowolnym remeshu |
| ADD `crates/engine/render-npr/tests/motion_tests.rs` | Stable/redraw/fade/LOD, zegary, wiele obiektów/widoków | `rtk cargo test -p amigo-render-npr --test motion_tests` | Nie porównywać odmiennych historii jak jednego stateless wejścia |
| MODIFY `plugins/gfx/npr-playground/tests/control_tests.rs` | Metadane, UI/Rhai, scope, presets, undo i niezależne kontrolki | P | Nie uznać samej obecności pola za dowód zachowania |
| MODIFY `plugins/gfx/npr-playground/tests/waterfall_tests.rs` | Granice Update/RenderExtract i własność domeny | `rtk cargo test -p amigo-npr-playground-plugin --test waterfall_tests` | Bez wymagań zależnych od prywatnych szczegółów WGPU |
| MODIFY `plugins/gfx/npr-playground/tests/diagnostics_tests.rs` | Rejestracja i znaczenie nowych `npr.*` | `rtk cargo test -p amigo-npr-playground-plugin --test diagnostics_tests` | Bez pełnego logowania per frame |
| MODIFY `crates/apps/app/src/tests/scene_loading_tests/threed.rs` | Rozszerzenie obecnych `npr_*` offscreen o Suzanne i sekwencje | `rtk cargo test -p amigo-app npr_` dopiero po owner tests | Nie migrować logiki domenowej do app tests/support |
| MODIFY `mods/npr-playground/tests/golden/README.md` | Ujęcia, sekwencje, wersje, seed, tolerancje i jawna regeneracja | D; test offscreen | Nie automatycznie zatwierdzać zmienionych obrazów |
| ADD `mods/npr-playground/tests/golden/suzanne-ear-512.png` | Nowa referencja po uruchomieniu renderera i przeglądzie przez człowieka | Test offscreen bez flagi aktualizacji | Nie generować referencji obrazem AI |
| MODIFY `plugins/gfx/npr-playground/README.md` | Zachowanie kontrolek i ograniczenia jakości | D | Bez deklaracji niezaimplementowanych funkcji jako gotowych |
| MODIFY `plugins/gfx/npr-playground/docs/pipeline.md` | Nowy przepływ powierzchni i historii domenowej | D | Bez dublowania pełnej teorii z tego dokumentu |
| MODIFY `plugins/gfx/npr-playground/docs/contributions.md` | Jawne kontrakty/snapshot i bridge | D | Bez nowych obowiązków app |
| MODIFY `plugins/gfx/npr-playground/docs/diagnostics.md` | Znaczenie debugów i liczników | D | Bez surowych dumpów packetów |

## 18. Etapy i warunki zakończenia

### S0. Ustalenie źródła artefaktów i regresji

READ operacje z rozdziału 3 oraz ADD pierwszych testów z 17.4. Zamrożone ujęcie
ucha: Final, FeatureClasses, StrokeIds; kolejno wyłączone hatch, creases,
humanizacja i grain. Zapis ustawień, rzeczywistej kamery, wersji modelu i seeda.
Test thin-shell wykazuje lub wyklucza błędny wybór start face. Test wielu packetów
wykazuje lub wyklucza zależność retencji od liczby obiektów.

Bramka: potrafimy wskazać, które rodziny linii tworzą siatkę w konkretnym capture.
Nie rozpoczynamy od podnoszenia wobble ani od zmiany wszystkich presetów.

### S1. Poprawne ścieżki i trwała tożsamość powierzchniowa

ADD/MODIFY `surface.rs`, `hatching.rs`, `field.rs`, `frame.rs`, `lod.rs` i testów
zgodnie z 17.1–17.2. Usunięcie hasha końców dopiero po migracji użytkowników.

Bramka: rigid motion nie zmienia kotwic i source IDs hatchy; ścieżki pozostają
na prawidłowej ściance; bank nie jest integrowany ponownie wskutek samej kamery.
To pierwszy etap, który może realnie usunąć przerysowywanie kreskowania.

### S2. Stabilny charakter sylwetki i publiczne sterowanie ruchem

MODIFY `stroke.rs`, `temporal.rs`, `gesture.rs`, `tessellation.rs` oraz plugin/UI
według 17.2–17.3. Split/merge, zegar widoku, niezależny fade i tryb Stable.
Celowy RedrawOnMotion powstaje na działającej stabilizacji, nie przez pozostawienie
niestabilnych hashy jako „efektu”.

Bramka: wyłączenie przerysowywania działa podczas obrotu i kamery; fade=0 nie
wyłącza trackera; sześć obiektów nie zmienia czasu retencji jednego obiektu.

### S3. Powierzchnia i kontury bez widocznej triangulacji

ADD/MODIFY `subdivision.rs`, `contour.rs`, `feature.rs`, `stroke.rs`, importer
i ustawienia SurfacePolicy według 17.1 i 17.3. Najpierw kontrolowane proxy
i jego kontury, następnie pola zerowe z kontrolą błędu i zgodności depth.

Bramka: łuk małżowiny pozostaje czytelny w zbliżeniu i obrocie; wireframe źródła
nie wyznacza rytmu gestów; cube zachowuje ostre krawędzie i brak diagonali.

### S4. Wewnętrzne linie formy

ADD `curvature.rs` i `form_lines.rs`, MODIFY pole/selekcję według tabel.
Najpierw curvature confidence i suggestive contours; apparent ridges dopiero
po sprawdzeniu normalnych, pochodnych i kondycjonowania projekcji.

Bramka: płaszczyzna i kula nie wytwarzają siatki fałszywych fałd; realne fałdy
otrzymują selektywne, stabilne linie; wynik sprawdzony na różnych triangulacjach
tej samej referencyjnej powierzchni w zadanej tolerancji geometrycznej.

### S5. Integracja materiału, wydajność i odbiór galerii

MODIFY operacje wykonania/debug i offscreen z 17.3–17.4. Strojenie narzędzia
na już poprawnych ścieżkach; pomiar globalnych budżetów i pełnej sekwencji ruchu.

Bramka: brak crasha alokacji, brak duchów na uchu, kontrolowane p95 i zatwierdzone
obrazy/sekwencje. Sama kompilacja albo większa liczba kresek nie zamyka etapu.

## 19. Szczegółowe kryteria testowe

### 19.1. Geometria i analiza

- Płaszczyzna: brak crease na diagonali, krzywizna bliska zeru, ciągła ścieżka
  przez oba warianty triangulacji.
- Kula analityczna: `k1≈k2≈1/R`, malejący błąd z próbkowaniem, niska pewność
  kierunku głównego; poprawny kontur i brak wymuszonych suggestive contours.
- Cylinder analityczny: krzywizny `0` i `1/R`, stabilna orientacja pola.
- Saddle/torus: obszary zmiany znaku krzywizny i narodziny linii zależnych od widoku.
- Cube/wedge: prawdziwe załamania zachowane; diagonale nie są cechą rysunkową.
- Dwie bliskie ścianki: wszystkie kotwice należą do właściwej warstwy, niezależnie
  od kolejności ścian w buforze.
- Mesh z UV seam: brak sztucznego boundary po poprawnym odtworzeniu połączeń;
  prawdziwa otwarta granica pozostaje granicą.
- Degenerate/non-manifold: jawna diagnoza, skończony czas, brak NaN i pętli.

Zmiana triangulacji może zmienić aproksymowaną powierzchnię, szczególnie po
subdivision. Porównujemy do tej samej znanej powierzchni albo kontrolujemy jej
różnicę; nie wymagamy niemożliwej niezmienności arbitralnego nowego mesha.

### 19.2. Ruch i niezmienniki danych

- Kamera w orbicie, model nieruchomy: niezmienny bank i source IDs hatchy.
- Obrót/przesunięcie modelu: niezmienne lokalne kotwice; poprawne world positions.
- Zoom in/out i resize: istniejące ślady odzyskują te same ID oraz parametr.
- Dodanie obiektu o wcześniejszym ID: nie zmienia seeda pozostałych.
- Dwa widoki: brak wzajemnego zużywania czasu i przenoszenia konturów.
- 1 versus 6 obiektów: identyczna retencja danego obiektu przy tym samym czasie.
- Dwa extracty tego samego frame: bez podwójnego fade i zmiany wariantu.
- Stable, nieruchomy input, fade=0: packet deterministyczny dla zamrożonego stanu.
- RedrawOnMotion: brak nowych wariantów na postoju, warianty niezależne od render FPS.
- ON→OFF: brak skoku do innego bazowego gestu, kamera nadal reaguje.
- Fade=0 versus fade>0: te same ścieżki i ID; różni się wyłącznie waga przejścia.
- Zasłonięcie: poprawna maska w tej klatce, bez opóźnionego ghostingu.

### 19.3. Miary i proponowane tolerancje

To wartości startowe do kalibracji, nie twierdzenia z literatury:

- Residuum kotwicy względem przypisanej ściany: maksymalnie `1e-5 * L` po
  normalizacji skali w testach; ostrzejsze przypadki ustalają tolerancję per test.
- Bezstratność source IDs istniejących hatchy przy samym rigid motion: 100%.
  ID aktywnego podzbioru LOD i view-dependent konturów oceniamy osobno.
- Aproksymacja krzywej: docelowo maks. 0,5 px w 512×512 przy wyłączonym wobble,
  mierzone wobec gęstszej referencji, nie tylko odległości jej wierzchołków.
- Tracking: raport p50/p95 błędu dopasowanych próbek; punkt wyjścia p95 ≤ 1 px
  dla niewielkiego kroku kamery w określonej sekwencji. Osobliwości są oznaczone.
- Byte budget: nigdy ponad zadeklarowany limit; overflow i brak rezerwacji
  zwracają błąd/degradację jakości z diagnostyką, nie panic alokacji GPU.

Różnica obrazów `I(t+1)-I(t)` nie jest samodzielną miarą migotania, bo obraz
powinien się zmieniać w ruchu. Dla kreskowania porównujemy po reprojekcji tych
samych kotwic, wykluczając disocclusion, granice visibility i przejścia LOD.
Osobno mierzymy odchylenie gestu względem nowej linii geometrycznej dla konturów.

### 19.4. Obrazy i sekwencje

Statyczne goldeny pozostają w istniejącym mechanizmie WGPU offscreen, ale ich
obraz nie jest jedynym kontraktem CI. Każdy scenariusz blokuje najpierw
wersjonowany `NprRenderPacket::fingerprint()` — CPU-side packet przekazany do
backendu, razem z geometrią stroke'ów, materiałem i statystykami. Dopiero potem
uruchamia się jawny przegląd PNG (`AMIGO_VERIFY_NPR_GOLDEN=1`), ponieważ
rasteryzacja i antyaliasing mogą zależeć od GPU. Nowe ujęcie ucha ma 512×512,
jawny seed i zapisany camera target/orbit/distance/FOV; wartości kamery dobieramy
po pierwszym capture, nie wymyślamy ich jako już zweryfikowanych. Porównujemy
osobno maskę konturu, depth/visibility i finalny materiał.

Sekwencja odbioru: pełny obrót modelu, orbit kamery, powolny zoom w obie strony,
przejście przez threshold LOD, zatrzymanie, zasłonięcie ucha i resize. Ruch
odtwarzamy przy renderowaniu 30/60/144 FPS z tą samą osią czasu symulacji.

Do ręcznego odbioru zestawiamy: źródłowy wireframe, neutralne linie bez materiału,
sam hatch, Final Stable, Final RedrawOnMotion. Oceniający ma sprawdzić, czy rysunek
opisuje fałdę ucha, czy nadal ujawnia mesha, czy styl pozostaje czytelny w ruchu.
Nie zatwierdzamy automatycznie nowego PNG po każdej zmianie algorytmu.

### 19.5. Kolejność uruchamiania walidacji

Po zmianie domeny najpierw `rtk cargo check -p amigo-render-npr`, potem odpowiedni
nowy suite. Dopiero po przejściu owner tests sprawdzamy zależne API/plugin/WGPU.
Istniejące testy końcowe:

```powershell
rtk cargo test -p amigo-app npr_playground_offscreen_matches_packet_contract
rtk cargo test -p amigo-app npr_pencil_profile_uses_depth_occluders_without_color_bands
rtk cargo test -p amigo-app npr_pencil_cylinder_streamlines_match_reviewed_golden
rtk cargo run -p amigo-plugin-check -- validate plugins/gfx/npr-playground
```

Nie uruchamiamy domyślnie `cargo test --workspace`, globalnego formatowania ani
aktualizacji goldenów. Nowe testy wymagają sprawdzenia, że rzeczywiście zostały
uruchomione — zielony wynik z zerem testów nie jest walidacją funkcji.

## 20. Wiedza źródłowa i granice projektu

Najważniejsze referencje są podlinkowane bezpośrednio przy odpowiadających im
mechanizmach. Korzystamy z definicji i publikacji autorów, a nie z przypadkowych
shaderów udających te algorytmy. Kod `rtsc` na stronie
[projektu suggestive contours](https://gfx.cs.princeton.edu/proj/sugcon/)
jest udostępniany jako GPL; plan nie przewiduje kopiowania go do repozytorium.
Ewentualną nową zależność wybieramy osobno, z przeglądem licencji i architektury.

Poza tym zakresem: rozpoznawanie anatomii, generatywne domyślanie się brakujących
fałd, malowanie ręcznych akcentów w edytorze, skinning/remeshing z przenoszeniem
rysunku, animacja kolejności powstawania rysunku i przebudowa ogólnego UI silnika.
Typy powierzchni i wspólny metadata contract pozostawiają miejsce na te rozszerzenia.

Najważniejsza kolejność decyzji: **poprawna powierzchnia i ścieżka → sensowna
selekcja linii → trwała tożsamość i parametr → kontrolowany gest → ślad materiału**.
Kolejność wdrażania S0–S5 pozwala naprawić ruch istniejącego kreskowania wcześniej,
bez uzależnienia pierwszego widocznego efektu od kompletnego estymatora krzywizny.
