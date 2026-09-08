# NPR: od modelu 3D do wiarygodnego rysunku

Status: **projekt do wdrożenia**, nie opis gotowych funkcji. Audyt kodu: 2026-09-08.

Uszczegółowienie: [linie formy i stabilność rysunku podczas ruchu](npr-surface-coherence.md).
Podlinkowana specyfikacja rozwija teorię powierzchni i kontrolę przerysowywania podczas obrotu
lub ruchu kamery oraz aktualizuje audyt tej części pipeline'u. W tym zakresie
jego kontrakty, etapy i kryteria odbioru mają pierwszeństwo przed poniższym planem.

Zakres: rozwój `amigo-render-npr`, wykonania WGPU i warsztatu `npr-playground`.
Pierwszy cel wizualny: monochromatyczny rysunek grafitowy, z czytelną bryłą,
kreskowaniem prowadzonym po formie, selektywnym konturem i śladem narzędzia na
papierze. Tusz korzysta później z tej samej organizacji rysunku, ale z własnym
modelem narzędzia i materiału. `ComicInk` pozostaje świadomym stylem komiksowym.

Wzory poniżej są propozycjami aproksymacji do strojenia, chyba że wyraźnie wskazano
konkretny model z publikacji. Pseudokod nie jest istniejącym API ani kodem gotowym
do wklejenia. Nazwy nowych plików i symboli oznaczają planowane operacje.

## 1. Decyzja przewodnia

Rysunek jest interpretacją bryły za pomocą wybranego medium. Potrzebujemy trzech
osobnych odpowiedzi:

1. **Co pokazać?** Forma, światło, kontakt, ważne krawędzie, wybrane szczegóły.
2. **Jak poprowadzić ślad?** Kierunek, długość, rytm, nacisk, zakończenia, poprawki.
3. **Co narzędzie zostawia na papierze?** Pokrycie, ziarnistość, miękkość, nasycenie.

Losowość działa dopiero wewnątrz tych decyzji. Nie zastępuje ani informacji o
kształcie, ani kompozycji walorowej. Powiększenie `wobble` przy obecnym wypełnieniu
nie wystarczy do zmiany języka obrazu.

Nie próbujemy zapisać jednej „całej teorii rysunku” jako uniwersalnego prawa.
Rysunek techniczny, akademicki ołówkowy, szkic poszukujący i ilustracja tuszem
mają różne cele. Silnik powinien oferować mechanizmy oraz jawne profile, a nie
zaszywać jeden gust w backendzie.

### 1.1. Zasady rysunkowe i ich konsekwencje obliczeniowe

| Zasada wybranego stylu | Co daje widzowi | Mechanizm |
| --- | --- | --- |
| Duże masy waloru przed detalem | Bryła czytelna w miniaturze | Pole docelowego tonu, ograniczony zakres jasności, hierarchia detalu |
| Kreska opisuje kierunek powierzchni | Poczucie objętości | Pole kierunków na powierzchni i ciągłe ścieżki |
| Różna ważność krawędzi | Skupienie uwagi | Osobna ocena znaczenia i widoczności linii |
| Krawędzie zgubione i odzyskane | Łączenie formy z tłem bez obrysowania wszystkiego | Kontrolowane wygaszanie fragmentów konturu |
| Ślad ma początek, przebieg i koniec | Wrażenie gestu | Asymetryczny profil nacisku, prędkości i oderwania narzędzia |
| Kolejne przejścia budują ciemność | Wiarygodny charakter medium | Warstwy śladów i model depozycji, nie tylko zmiana koloru |
| Papier pozostaje widoczny | Charakter grafitu i zakres półtonów | Pokrycie zależne od mikrostruktury podłoża |
| Uproszczenie zależy od skali | Brak szumu po oddaleniu | Selekcja i LOD kreskowania zamiast zagęszczania bez limitu |

To założenia projektowanego profilu, nie lista reguł obowiązujących wszystkich
artystów. Badanie [Where Do People Draw Lines?](https://gfx.cs.princeton.edu/proj/ld3d/lineset/index.html)
jest istotną referencją: analizuje zgodność linii artystów z własnościami modeli,
ale nie sprowadza wszystkich decyzji rysunkowych do jednego rodzaju krawędzi.

## 2. Punkt wyjścia wcześniejszego audytu

Poniższe obserwacje dotyczą odczytanych źródeł, nie pomiarów wydajności ani oceny
aktualnego zrzutu ekranu. W tej pracy nie wykonano nowej sesji interaktywnej.

Tabela zachowuje punkt wyjścia sprzed późniejszych zmian w worktree, nie jest
inwentaryzacją obecnego wdrożenia. Aktualizację dotyczącą powierzchni, hatching,
LOD i historii zawiera [audyt stabilności](npr-surface-coherence.md#3-stan-kodu-i-ryzyka-do-potwierdzenia-testami).

| Miejsce | Stan obecny | Konsekwencja dla rozbudowy |
| --- | --- | --- |
| `crates/engine/render-npr/src/frame.rs`, `build_packet_with_topology` | Iloczyn normalnej ściany i światła wybiera `shadow`, `mid` albo `light` | Obraz ma bazową strukturę trzech pasm cel-shadingu |
| Ten sam plik, `append_hatching` | Kreskowanie generowane osobno w rzutowanych trójkątach | Ścieżki nie mają ciągłości powierzchniowej; koszt i charakter śladu zależą od triangulacji |
| `feature.rs` | Boundary, Silhouette, Crease | Brak linii opisujących gładką formę; hatch nie powinien udawać crease |
| `stroke.rs`, `chain_features` | Łączenie topologiczne; ID związane z krawędziami łańcucha | Powtarzalność jednego wejścia nie zapewnia stabilności przy zmianie sylwetki |
| `gesture.rs`, `sample` | Deterministyczne składowe sinusoidalne i próbkowany szum | Początek mechanizmu gestu, ale jeszcze bez trwałej parametryzacji śladu |
| `tool.rs`, `ToolResponse` | Nacisk, szerokość, grain, `edge_softness` | Miękkość nie dochodzi do faktycznego renderowania śladu |
| `tessellation.rs`, `tessellate_polyline_variant` | Quady oraz pełne koła z 12 trójkątów przy próbkach | Nadmiar geometrii i nakładające się fragmenty alfa mogą przyciemniać połączenia |
| `render-wgpu/.../renderer/npr.rs` | Shader kreski jest aliasem shadera fill; kolor × coverage | Brak lokalnych współrzędnych śladu i materiału oddziałującego z papierem |
| `render-wgpu/.../render/world.rs`, `render_npr_commands` | Indeksy domenowe rozwijane do listy wierzchołków | Tracimy korzyści z indeksowanego mesha |
| `geometry.rs`, `icosphere` | Bazowy dwudziestościan bez subdivide | To nie jest dostatecznie gładka kula do oceny subtelnego cieniowania |
| `topology.rs`, `build_topology` | Krawędź przechowuje najwyżej dwie ściany | Domena potrzebuje jawnej walidacji non-manifold, a nie cichego pomijania kolejnych ścian |
| `plugins/gfx/npr-playground/src/render/mod.rs`, `rebuild` | Cache geometrii/topologii; porównanie całych ustawień i viewportu | Obrót unieważnia packet; przygotowanie powierzchni warto oddzielić od pracy każdej klatki |

### 2.1. Istniejące zabezpieczenia pamięci

W worktree są już limit 32 000 hatchy na packet, clipping hatchy do viewportu
oraz dzielenie uploadu WGPU na porcje najwyżej 64 MiB, ograniczone limitem device.
To zabezpieczenia, których nie wolno zgubić podczas migracji.

Nie dowodzą jednak, że całkowity koszt klatki jest mały:

- Limit jest per obiekt/packet, nie globalny dla galerii.
- Przerwanie pętli po kolejnych ścianach faworyzuje kolejność trójkątów.
- Chunks nie usuwają zbiorczej alokacji CPU ani łącznej rezydencji GPU.
- Ukryte powierzchnie mogą kosztować planowanie i tessellację przed depth testem.

Zgłoszony wcześniej błąd alokacji 460 259 520 B przy limicie 268 435 456 B jest
konkretnym wymaganiem regresyjnym. Bez odtworzenia tej konfiguracji nie przypisujemy
mu jednej udowodnionej przyczyny ani nie deklarujemy go ostatecznie zamkniętym.

## 3. Architektura docelowa

```text
model + jawne cechy powierzchni + profil rysunku + kamera + światło
    -> przygotowana powierzchnia: topologia, normalne, pola, struktury zapytań
    -> analiza widoku: widoczność, ton docelowy, kandydaci linii
    -> plan rysunku: role, ścieżki, hierarchia, gęstość, budżet
    -> gesty: parametryzacja, nacisk, odchylenie, ewentualna poprawka
    -> ślady narzędzia: szerokość, kontakt, parametry materiału
    -> neutralny packet: geometria zasłaniająca + ślady + materiały + diagnostyka
    -> WGPU: rasteryzacja, test głębokości, depozycja, kompozycja
```

Własność odpowiedzialności:

- `amigo-render-npr`: algorytmy powierzchni, tonu, wyboru i prowadzenia linii,
  styl, gest, tessellacja, jawny stan temporalny. Bez typów WGPU.
- `amigo-render-api`: kontrakt przekazania wyniku i diagnostyk renderowania.
- `amigo-render-wgpu`: wykonanie zadeklarowanych materiałów i passów, zasoby GPU.
- Plugin: stan sceny, metadane kontrolek, wybór profilu, Update i RenderExtract.
- Bundles: istniejący bridge extractora. Bez powtórnej ekstrakcji i stylowania.
- Mod: scena, modele, układ panelu i Rhai. Bez implementacji algorytmów renderera.
- App: uruchomienie, okna, podanie klatki. Bez decyzji, jak rysować ołówkiem.

Nie dodajemy drugiego renderera szkicu obok obecnego NPR. Rozbudowujemy wspólny
pipeline i migrujemy jego kontrakt. Styl komiksowy jest jednym z profili tego
pipeline'u; nie staje się ukrytą ścieżką kompatybilności.

### 3.1. Proponowane typy domenowe

```rust
// Pseudokod: nazwy docelowych odpowiedzialności, nie stabilne ABI.
struct DrawingProfile {
    tone: TonePolicy,
    lines: LinePolicy,
    hatching: HatchingPolicy,
    gesture: GesturePolicy,
    tool: ToolProfile,
    paper: PaperProfile,
    quality: QualityBudget,
}

enum StrokeRole {
    StructuralContour,
    InternalForm,
    Hatch,
    CrossHatch,
    Construction,
    Accent,
}

struct PlannedStroke {
    id: StrokeId,
    role: StrokeRole,
    source_feature: Option<FeatureClass>,
    path: SurfaceOrViewPath,
    gesture: GestureParameters,
    material: MaterialId,
    priority: f32,
}

struct SurfaceAnchor {
    instance: ObjectInstanceId,
    primitive: PrimitiveId,
    triangle: TriangleId,
    barycentric: Vec3,
}

struct PathSample {
    anchor: SurfaceAnchor,
    path_coordinate: f32, // trwała współrzędna oryginalnej ścieżki
    world_position: Vec3,
}
```

`FeatureClass` opisuje przyczynę geometryczną. `StrokeRole` opisuje zadanie
rysunkowe. `ToolProfile` opisuje narzędzie. Hatch nie dostaje `Crease` tylko po to,
żeby skorzystać z istniejącej szerokości. Dwa egzemplarze tego samego mesha mają
różne `ObjectInstanceId`. ID nie zależy od indeksu obiektu w widocznym `Vec`.

Nazwy `PencilStudy`, `InkStudy`, `ComicInk` oznaczają konstruktory typed profili.
YAML może przechowywać wersjonowane wartości presetu i layout, ale nie program
stylowania. Nie dokładamy kolejnych pól ołówka do typu nazwanego `ComicInk`.

## 4. Powierzchnia: trzeba wiedzieć więcej niż gdzie są trójkąty

### 4.1. Przygotowanie wykonywane raz na rewizję mesha

Proponowany `NprPreparedSurface` przechowuje:

- pozycje, indeksy, trwałe identyfikatory primitive/triangle;
- sąsiedztwo, granice, szwy, orientację, jawne ostre krawędzie;
- normalne geometryczne oraz osobno normalne do analizy gładkiej powierzchni;
- obszary gładkości i ograniczenia prowadzenia ścieżek;
- pole kierunków z miarą wiarygodności; krzywiznę dopiero w odpowiednim etapie;
- strukturę przyspieszającą zapytania o przecięcie i widoczność, jeśli włączona;
- diagnostykę jakości wejścia i klucz rewizji.

Normalna ściany pozostaje właściwa dla orientacji i cullingu. Normalna interpolowana
jest właściwa dla zamierzonego gładkiego cieniowania. Uśrednianie normalnych przez
ostrą krawędź sześcianu błędnie zaokrągli jego formę.

Importer nie może bezwarunkowo scalać wszystkich współrzędnych o tej samej
pozycji: szew materiału, osobne nałożone powierzchnie i ostra krawędź mogą być
zamierzone. Polityka weld/smoothing musi być jawna i objęta testem.

Przygotowanie zgłasza indeksy poza zakresem, NaN, trójkąty zerowego pola,
niepoprawny winding i non-manifold. Możliwa świadoma polityka „rysuj jako otwartą
powierzchnię” jest częścią wejścia, nie cichą naprawą w backendzie.

### 4.2. Modele testowe

Sześcian sprawdza strukturę, brak diagonali i ostre krawędzie. Nie wystarcza do
oceny rysunkowej miękkości. Potrzebujemy również gładkiej kuli, cylindra,
powierzchni siodłowej, torusa i modelu organicznego z wnękami. Dla kuli dodajemy
rzeczywistą subdivizję albo osobną poprawnie nazwaną geometrię, bez udawania,
że obecne 20 ścian reprezentuje gładką bryłę.

## 5. Ton: co ma być jasne, a co ciemne

### 5.1. Oddzielenie oświetlenia od sposobu pozostawienia śladu

Najpierw wyznaczamy pole docelowej jasności. Dopiero później wybieramy liczbę,
grubość i rodzaj kresek potrzebnych do jego osiągnięcia.

Przykładowy model początkowy, w liniowej przestrzeni światła:

```text
E(x) = kd * max(dot(Ns(x), L), 0) * Vlight(x)
     + ambient * Aambient(x)

Rtarget(x) = value_curve(E(x), artistic_focus, material_tone)
Rtarget   = clamp(Rtarget, Rblack, Rpaper)
```

`Ns` to normalna analityczna/interpolowana. `Vlight` oznacza widoczność źródła
światła, a `Aambient` tłumienie światła otoczenia. Nie mnożymy całego światła przez
AO: cień rzucany i occlusion otoczenia to różne składniki. Również widoczność dla
kamery nie jest widocznością światła.

`value_curve` grupuje walory i zachowuje czytelność, ale w profilu ołówkowym nie
powinna narzucać trzech jednolitych kolorów ścian. Możemy zostawić szeroką jasną
masę i skupić ciemne akcenty w niewielu miejscach. To jawna decyzja profilu.

Pierwszy prototyp tonu może używać tylko rozproszonego światła i jawnie deklarować
brak cieni rzucanych. Następny dodaje referencyjne zapytania promieniowe/BVH po
stronie domeny. Ewentualne późniejsze przyspieszenie GPU wymaga osobnego kontraktu
zapytania, a nie odczytywania zamiaru z obecności mesha.

### 5.2. Ton nie musi być kolorem fill

Dla `PencilStudy` tryb podstawowy:

```text
jasny papier + lokalne ślady grafitu + ewentualna bardzo delikatna warstwa podkładu
```

Depth-only bryła nadal istnieje. Usunięcie kolorowego fill nie może usunąć geometrii
zasłaniającej tylne linie. Opcjonalny podkład „bokiem ołówka” jest zadeklarowanym
materiałem o niskiej częstotliwości, a nie przywróceniem trzech pasm pod inną nazwą.

### 5.3. Kalibracja pokrycia zamiast arbitralnego mnożnika density

Użyteczna, uproszczona krzywa nasycenia:

```text
R(M) = Rblack + (Rpaper - Rblack) * exp(-sigma * M)
Mtarget = -ln(clamp((Rtarget-Rblack)/(Rpaper-Rblack), eps, 1)) / sigma
```

`M` to umowna ilość materiału na jednostkę powierzchni obrazu. To aproksymacja
monotonicznego ciemnienia, nie fizyczna symulacja optyki grafitu. Nie obiecujemy,
że etykieta HB odpowiada rzeczywistej twardości bez kalibracji próbkami.

W praktyce tworzymy tablicę ze swatchy: szerokość, odstęp, nacisk, liczba warstw,
papier -> zmierzona średnia luminancja i kontrast. Planner odwraca tę zależność.
Zmiana narzędzia nie powinna przypadkowo zamieniać wszystkich półtonów w czerń.

Kontrola tonu to odrębny problem od samego narysowania hatchy; przydatne punkty
odniesienia to [Real-Time Hatching](https://gfx.cs.princeton.edu/proj/hatching/)
oraz [Fine Tone Control in Hardware Hatching](https://gfx.cs.princeton.edu/pubs/Webb_2002_FTC/index.php).
Proponowany tutaj planner ścieżek nie jest implementacją tonal art maps.

### 5.4. Światło formy, cień formy i cień rzucany

W projektowanym studium warto rozróżniać kilka informacji, nawet jeśli pierwszy
algorytm realizuje tylko część z nich:

- Strona oświetlona i półton opisują zmianę orientacji powierzchni.
- Przejście do cienia formy wynika z odwrócenia powierzchni od źródła światła.
- Światło odbite może rozjaśniać fragment cienia; nie jest białą kreską doklejoną
  do każdego konturu. W MVP można go nie modelować i jawnie opisać ograniczenie.
- Cień rzucany informuje o zasłonięciu źródła przez inną część sceny.
- Kontakt i ciasne szczeliny mogą uzasadniać ciemne akcenty, ale potrzebują danych
  o relacji powierzchni, nie samej pozycji `y` na ekranie.
- Refleksy kierunkowe zależą od materiału i światła. Dla pierwszego matowego
  studium grafitowego możemy uprościć reflektancję modelu, żeby izolować bryłę.

Nie każda bryła ma wyraźny „pas rdzenia cienia” w dowolnym oświetleniu. Nie
powinniśmy wymuszać gotowego schematu jasność-ciemność-jasność na każdym normalnym.
To model światła dostarcza strukturę, a profil rysunkowy decyduje o jej selekcji.

### 5.5. Kompozycja i poziom abstrakcji

Poprawne lokalne kreski mogą nadal tworzyć słaby rysunek jako całość. Potrzebujemy
kontroli trzech skal: masy bryły, grupy śladów, pojedynczego śladu. Podgląd mocno
pomniejszonego obrazu pomaga ocenić pierwszą, a swatch w powiększeniu trzecią.

Jawna mapa skupienia uwagi może zwiększać detal i kontrast w wybranym regionie
oraz ograniczać je poza nim. Początkowo wybór regionu jest autorski lub pochodzi
z zaznaczenia obiektu. Nie dokładamy niezweryfikowanej „AI oceniającej ważność”
ani heurystyki `nazwa zawiera face`.

Uproszczenie może scalać pobliskie podobne ślady i usuwać nieistotne detale, ale
nie powinno zmieniać topologii widocznego obiektu. Przerwy w konturze i redukcja
kreskowania mają osobne zakresy — nie chcemy jednocześnie usunąć wszystkich
wskazówek kształtu w tym samym miejscu.

## 6. Linie: wybór i hierarchia przed deformacją

### 6.1. Co może stać się linią

Pierwszy zakres: granice, kontury zasłaniające i świadomie wybrane ostre krawędzie.
Kolejny: linie gładkiej formy, wybrane grzbiety/doliny i suggestive contours.
Jeszcze później: autorskie linie konstrukcyjne i akcenty wskazujące punkt skupienia.

Nie rysujemy automatycznie każdej dużej krzywizny. Drobny szum geometrii może dać
więcej kandydatów niż informacji. Kandydat potrzebuje oceny znaczenia w obrazie,
minimalnego rozmiaru i wiarygodności danych.

Przykładowa funkcja priorytetu do kalibracji:

```text
importance = role_weight
           * visibility_confidence
           * projected_extent_weight
           * local_contrast_weight
           * authored_focus_weight
```

Mnożenie jest propozycją, nie jedyną poprawną postacią. Kluczowe jest, by każdy
składnik był jawny, wizualizowalny i miał sens dla danej roli.

### 6.2. Lost-and-found edges

Na jasno oświetlonej stronie obiektu można osłabić fragment konturu, jeżeli forma
pozostaje czytelna dzięki sąsiednim walorom i liniom. Przy kontakcie, zasłonięciu
lub istotnym załamaniu można go wzmocnić. Nie ustalamy reguły „wszystkie dolne
krawędzie są grube”: orientacja na ekranie nie opisuje kontaktu ani światła.

```text
visibility_gate = geometric_visibility()  // twardy warunek bezpieczeństwa
edge_strength   = artistic_edge_policy(local_context)
line_coverage   = visibility_gate * smooth_along_path(edge_strength)
```

Wygaszanie artystyczne jest płynne wzdłuż ścieżki; widoczność geometryczna nadal
uniemożliwia przeciekanie przez obiekt z przodu. Przerwy wynikające ze stylu nie
powinny być równomiernie rozrzuconymi dziurami.

### 6.3. Suggestive contours dopiero po dobrych normalnych i krzywiźnie

To konkretna rodzina linii, odmienna od crease i ridge. W lokalnym opisie wykorzystuje
zerowanie krzywizny radialnej i odpowiedni warunek jej zmiany w kierunku widoku.
Implementacja wymaga pełnych warunków publikacji, stabilnych pochodnych i filtrów,
nie tylko testu `abs(curvature) < epsilon`.

Referencje i ostrzeżenia o jakości siatki znajdują się na stronie
[Suggestive Contours](https://gfx.cs.princeton.edu/proj/sugcon/index.html).
To etap późniejszy: aktualny niski poziom szczegółowości kuli nie daje wiarygodnej
podstawy do strojenia takich linii.

## 7. Kierunek kreski: pole na powierzchni

### 7.1. Dlaczego nie jeden kąt ekranowy

Kreskowanie może opisywać przekrój bryły: na cylindrze obiegać jego obwód, na
policzku sugerować wypukłość, na płaskiej ścianie być prawie proste. Stały kąt na
całym obrazie jest dopuszczalnym stylem, ale nie powinien być jedynym mechanizmem.

Proponowane źródła pola, wybierane jawnie:

1. Autorskie kierunki powierzchni/regionów.
2. Projekcja ustalonego kierunku obiektu na płaszczyznę styczną.
3. Kierunki wynikające z krzywizny, kiedy oszacowanie jest wiarygodne.
4. Ustalony kierunek strony dla świadomie płaskiego kreskowania.

Dla demonstratorów dopuszczamy analityczne pole kuli/cylindra zadeklarowane przy
budowie geometrii. Backend nie rozpoznaje modelu po nazwie `sphere` czy `cylinder`.

### 7.2. Kierunek bez zwrotu i miejsca osobliwe

Hatch o kierunku `d` i `-d` ma tę samą orientację. Naiwne uśrednianie tych wektorów
daje zero. W jednej bazie stycznej można wygładzać reprezentację
`(cos(2*theta), sin(2*theta))`, a między sąsiednimi ścianami trzeba uwzględnić zmianę
bazy/transport. Samo uśrednienie kątów świata nie wystarcza.

Na idealnej kuli krzywizny główne są równe, więc nie ma wyróżnionego kierunku
głównego. W pobliżu takich miejsc wybieramy jawny kierunek artystyczny lub
kontynuację pola z kontrolowaną wiarygodnością. Nie generujemy szumu z niestabilnych
wektorów własnych. Osobliwości pola są normalną częścią problemu: ścieżka może
tam zakończyć się lub zmienić region.

### 7.3. Ścieżki przekraczają krawędzie triangulacji

```rust
fn trace_hatch(surface, seed_anchor, field, limits) -> SurfacePath {
    let mut path = Path::new(seed_anchor);
    let mut cursor = seed_anchor;
    let mut direction = field.at(cursor).choose_orientation();

    while path.length < limits.max_length && path.steps < limits.max_steps {
        // RK2: kierunek w środku kroku daje lepsze prowadzenie niż prosty Euler.
        let step = adaptive_surface_step(cursor, field, limits);
        let mid = walk_across_triangles(cursor, direction * (step * 0.5))?;
        let d_mid = field.at(mid).align_with(direction);
        let next = walk_across_triangles(cursor, d_mid * step)?;

        if hits_declared_boundary(next) || low_field_confidence(next) { break; }
        if too_close_to_existing_path(next) || detects_cycle(next) { break; }

        path.push(next); // zachowaj barycentrics i współrzędną ścieżki
        cursor = next;
        direction = d_mid;
    }
    path
}
```

Pełna implementacja obsługuje zakończenia na granicach i przejścia przez wierzchołki,
limit liczby odwiedzonych ścian, zerowy postęp i wartości niefinitywne. Wygładzamy
kierunek śladu, nie geometrię przez zamierzone ostre krawędzie.

## 8. Rozmieszczanie hatchy i budowanie półtonów

### 8.1. Trwała rodzina kandydatów

Zamiast losować komplet kresek dla każdej klatki przygotowujemy stabilnie
identyfikowanych kandydatów na powierzchni. Mogą powstawać z deterministycznego,
powierzchniowego próbkowania typu blue-noise i wielopoziomowego porządku dodawania.
Nie opieramy rozkładu wyłącznie na indeksie trójkąta i jego ekranowym bboxie.

Przy rosnącym tonie aktywny zbiór powinien zasadniczo zawierać dotychczasowy zbiór
oraz nowe ślady. To zapobiega wymianie całego wzoru po niewielkiej zmianie światła.
Wymaga zaprojektowanego progresywnego rozkładu: przypadkowy prefix listy punktów
nie gwarantuje dobrego rozmieszczenia przy każdym poziomie gęstości.

```text
required_mass = target_mass(region)
for candidate in progressive_candidates(region):
    gain = estimated_tone_gain(candidate, tool, paper)
    if gain <= 0: continue
    if current_mass < required_mass and budget.can_reserve(candidate.cost):
        schedule(candidate)
        current_mass += gain
```

Rzeczywista ocena tonu powinna działać lokalnie, np. na komórkach pola pokrycia,
bo jedna średnia dla dużego regionu nie odtworzy gradientu. Powyżej pokazano tylko
zasadę planowania. Koszt rezerwujemy przed budową dużych tablic wierzchołków.

### 8.2. Cross-hatching

Druga rodzina ma własną orientację, próg wejścia, długość, nacisk i ID przejścia.
Nie musi przecinać pierwszej pod kątem 90 stopni. Próg powinien mieć miękkie
przejście, aby ciemniejąca bryła nie nagle pokrywała się kratką.

Najpierw stroimy jedną rodzinę. Kolejną dodajemy, gdy wymagany ton byłby osiągany
przez zbyt gęste, sklejające się pierwsze ślady. Dodatkowe warstwy i ciemne akcenty
mają ograniczony udział, żeby nie zniszczyć hierarchii.

### 8.3. Skala i zoom

Kotwice są powierzchniowe, lecz czytelność śladu oceniamy w obrazie. Lokalny
Jacobian projekcji mówi, ile pikseli zajmuje przesunięcie w kierunku stycznym.
Pozwala ocenić odstęp, długość i zagęszczenie po projekcji, szczególnie przy
powierzchniach widzianych pod ostrym kątem.

Szerokość narzędzia pozostaje w zadeklarowanych jednostkach strony/pikselach,
a zoom zmienia wybór LOD i liczbę śladów. Wyższy LOD dodaje szczegół do istniejącego
wzoru. Przejścia mają histerezę i krótkie wygaszenie, nie skokowy nowy seed.

Nie da się jednocześnie utrzymać dosłownie tych samych kresek na powierzchni,
stałego odstępu ekranowego przy dowolnym zoomie i niezmiennej liczby kresek.
Wybrany kompromis musi być jawny: stabilne kotwice, stopniowa zmiana zestawu,
kontrolowana gęstość obrazu.

## 9. Humanizacja: gest jako uporządkowany proces

### 9.1. Trzy skale odchylenia

Rozdzielamy:

1. **Intencję gestu**: uproszczenie krzywej do czytelnego łuku, dłuższa fraza,
   skrócenie mniej istotnego fragmentu. Skala całego pociągnięcia.
2. **Niedokładność prowadzenia**: skorelowane odchylenie, łagodne przestrzelenie,
   zmiana nacisku, lokalna poprawka. Skala części pociągnięcia.
3. **Mikrostrukturę medium**: pory papieru, nierówny kontakt, drobna chropowatość
   brzegu. Skala fragmentu/piksela, obsługiwana przez materiał.

Nie tessellujemy osobnego trójkąta dla każdego ziarna papieru. Nie używamy tego
samego losowania do pozycji, nacisku i ziarnistości: prowadzi to do sztucznej
korelacji, np. każdy skręt kreski staje się automatycznie ciemną plamą.

### 9.2. Parametryzacja ścieżki

Przechowujemy współrzędną oryginalnej ścieżki przed clippingiem i ponownym
próbkowaniem. Dla powierzchni może to być długość w jednostkach obiektu, dla
konturu współrzędna przenoszona przez dopasowanie ścieżek. Dodatkowo liczymy
bieżącą długość po projekcji do oceny szerokości i jakości tessellacji.

Przy zoomie chcemy zachować fazę charakterystycznego łuku, ale nie powiększać
mikrodrżenia do kilkunastu pikseli. Dlatego faza gestu ma trwałą kotwicę, a jego
amplituda i pasmo widoczne na stronie mają osobną politykę skali. Gdy potrzebujemy
drobniejszej składowej, wprowadzamy ją płynnie jako kolejny poziom szczegółu.

### 9.3. Przykładowy model odchylenia

```text
offset(s) = end_envelope(s) * (
    intended_bend(s)
  + A_broad * smooth_noise(stroke_seed, s / lambda_broad, lane=0)
  + A_fine  * smooth_noise(stroke_seed, s / lambda_fine,  lane=1)
)

pressure(s) = clamp(
    pressure_envelope(s)
  + A_pressure * smooth_noise(stroke_seed, s / lambda_pressure, lane=2),
    0, 1
)
```

`smooth_noise` interpoluje stabilne wartości w węzłach dziedziny, a nie losuje
wartość na każdy wierzchołek tessellacji. Różne rozdzielczości tej samej ścieżki
odczytują tę samą funkcję. Węzły i długości korelacji nie zależą od numeru klatki.

Duża pewność gestu oznacza mniejsze błędy prowadzenia, mniej niepotrzebnych
poprawek i dłuższe czytelne odcinki. Nie oznacza stałej szerokości ani braku
zróżnicowania nacisku. Mała pewność nie oznacza białego szumu.

```rust
let stroke_seed = hash(profile_seed, object_id, path_id, pass_id);
let broad = noise(stroke_seed, Lane::Motion, path_coordinate);
let pressure = noise(stroke_seed, Lane::Pressure, path_coordinate);
let paper_contact = sample_paper(page_position); // wspólny papier, osobny proces
```

Haszujemy jawnie serializowane liczby o ustalonym formacie; nie adresy pamięci,
kolejność `HashMap` ani domyślny hasher z losowanym stanem.

### 9.4. Prędkość i minimum jerk

Pomocnicza funkcja przejścia dla gestu od punktu do punktu:

```text
q(t) = 10*t^3 - 15*t^4 + 6*t^5,  t w [0, 1]
```

Ma zerową prędkość i przyspieszenie na końcach. Może opisywać postęp lokalnego
pociągnięcia. Inspiracją jest model minimum jerk z badania
[Flash i Hogan, 1985](https://pubmed.ncbi.nlm.nih.gov/4020415/).
Badanie ruchów ramienia nie stanowi kompletnego modelu rysowania, chwytu ani
depozycji grafitu. Nie wyprowadzamy z niego uniwersalnej zależności nacisku od
prędkości i nie nazywamy pojedynczego wielomianu symulatorem artysty.

Dla gotowego statycznego śladu „czas gestu” jest parametrem wewnętrznym. Nie trzeba
symulować ręki w każdej klatce. Animacja ujawniania rysunku to oddzielny tryb,
korzystający z zapisanej kolejności i długości pociągnięć.

### 9.5. Początek, koniec, poprawka

Taper ma osobno długość wejścia, długość wyjścia i charakter oderwania narzędzia.
Odcinek może zacząć się twardo i zakończyć lekko; nie każdy ma być symetryczną
soczewką. Długości wyrażamy także w jednostkach strony, żeby krótka i długa kreska
nie miały identycznych proporcji końcówek z konieczności.

Poprawka jest nowym gestem na wybranym zakresie ścieżki. Ma własne ID przejścia,
nacisk i ograniczony wpływ na walor. Jej prawdopodobieństwo może zależeć od trudności
łuku i profilu, ale nie może oznaczać automatycznego podwojenia każdego konturu.
Warstwę konstrukcyjną udostępniamy tylko jako świadomy styl lub autorskie dane.

## 10. Narzędzie: ołówek, cienkopis, stalówka, pędzel

| Narzędzie | Główne zmienne | Oczekiwany ślad | Czego nie utożsamiać |
| --- | --- | --- | --- |
| Ołówek | Nacisk, promień końcówki, pochylenie, względna twardość, papier | Niepełne pokrycie, zróżnicowany brzeg, kumulacja grafitu | Ołówek to nie czarny wąż z losową alfą |
| Cienkopis | Rozmiar końcówki, niewielka reakcja na nacisk, przepływ | Dość stała szerokość, wyraźny brzeg | Stała szerokość nie wymusza idealnej ścieżki |
| Stalówka | Kształt kontaktu, kąt, ugięcie, przepływ | Zmiana szerokości zależna od kierunku i nacisku | Kąt stalówki nie jest kątem hatchingu |
| Pędzel | Nacisk, pochylenie, opóźnienie włosia, nasycenie | Szeroki zakres szerokości, smukłe końce, suche rozdzielenia | Pełna dynamika włosia nie jest potrzebna w pierwszym etapie |

### 10.1. Jedno źródło obliczenia szerokości

Dla eliptycznego śladu kontaktu o półosiach `a`, `b`, osiach jednostkowych `u`, `v`
i normalnej ekranowej do ścieżki `n`:

```text
half_width = sqrt((a * dot(n,u))^2 + (b * dot(n,v))^2)
```

To szerokość podpory elipsy w kierunku poprzecznym do ruchu. Nacisk i pochylenie
mogą zmieniać `a`, `b`. Nie mnożymy jej drugi raz przez niezależną heurystykę
`nib_aspect` w tessellatorze. Przy zakrętach i zmiennym kontakcie nadal potrzebne
są poprawne połączenia obwiedni.

Dla ołówka rozdzielamy pracę końcówką i bokiem. Zwiększenie nacisku może zwiększać
ilość materiału silniej niż szerokość. Dla cienkopisu reakcja szerokości może być
minimalna. Są to strojone modele narzędzi, a nie jeden wspólny mnożnik pressure.

### 10.2. Twardość i kalibracja

Początkowe `hardness` jest parametrem względnym: wpływa na nasycenie, kontakt
z ziarnem i reakcję na nacisk. Nazwy HB/2B mogą być nazwami inspiracji stylistycznej,
ale bez próbek nie oznaczają zgodności z konkretnym producentem i papierem.

Docelowo robimy własne skany lub zdjęcia próbek za zgodą autora: kilka nacisków,
dwa papiery, końcówka/bok, pojedyncze i wielokrotne przejście. Zapisujemy oświetlenie,
skalę i sposób normalizacji obrazu. Model rozwijamy na podstawie różnic względem
tych próbek, a nie na podstawie coraz większej liczby losowych suwaków.

## 11. Papier i depozycja materiału

### 11.1. Papier wpływa na ślad, nie tylko na tło

Potrzebujemy wspólnego pola mikrostruktury `height(page_xy)` i opcjonalnego kierunku
włókien. Przy lekkim kontakcie zostaje materiał głównie na wystających częściach;
przy silniejszym pokrycie rośnie. To kierunek modelowania zgodny z badaniami medium,
ale konkretna funkcja kontaktu poniżej jest naszą aproksymacją.
Referencja: [Observational Models of Graphite Pencil Materials](https://onlinelibrary.wiley.com/doi/abs/10.1111/1467-8659.00386).

```text
contact = smoothstep(threshold(pressure) - softness,
                     threshold(pressure) + softness,
                     paper_height(page_xy))

deposit = path_coverage * contact * material_load(pressure, tool)
```

Próg maleje przy zwiększaniu kontaktu. `softness` ma dolną granicę i uwzględnia
filtrowanie. Formuła wymaga kalibracji, bo same losowe wysokości nie opisują całej
mechaniki grafitu. Osobno można dodać teksturę samego śladu narzędzia w jego lokalnych
współrzędnych; nie zastępuje ona wspólnego papieru.

### 11.2. Układy współrzędnych

- **Powierzchnia obiektu**: kotwice hatchy, kierunki formy, stabilne szczegóły.
- **Ścieżka**: trwały parametr gestu oraz poprzeczna odległość od jego osi.
- **Strona/viewport**: szerokość narzędzia, struktura papieru, antyaliasing.

Papier nie obraca się z modelem. Wzór hatchy nie powinien płynąć po powierzchni jak
naklejony filtr ekranu. Podczas obrotu nowe miejsca śladu naturalnie kontaktują się
z innymi miejscami papieru. To zamierzona zmiana, odróżniona od losowej wymiany
całego ziarna co klatkę.

Kontrakt viewportu określa fizyczne piksele i skalę strony. Na zmianie DPI nie
możemy pomylić logical pixels panelu z pikselami targetu renderera. Tryb druku o
stałym fizycznym rozmiarze papieru może później mieć osobną skalę; MVP nie obiecuje
fizycznych milimetrów na dowolnym monitorze.

### 11.3. Brzeg i antyaliasing

Shader śladu potrzebuje m.in. odległości poprzecznej, szerokości, współrzędnej
wzdłuż ścieżki i parametrów materiału. Pokrycie można wyznaczyć z odległości
od idealnego śladu i szerokości filtra, np. przez `fwidth` oraz profil miękkości.
Quady muszą mieć margines na filtr i chropowatość: shader nie narysuje fragmentu
poza przyciętą geometrią.

Drobne ziarno filtrujemy zgodnie ze skalą; proceduralny szum bez pasma lub tekstura
bez właściwego LOD będzie migać. Przy oddaleniu zachowujemy średnie pokrycie,
zamiast próbować wyświetlić każde subpikselowe ziarno.

### 11.4. Depozycja a nakładanie trójkątów

Rozdzielamy dwa przypadki:

- Nakładanie fragmentów tessellacji **tego samego przejścia** nie powinno tworzyć
  ciemnej kropki przy każdym joinie.
- Drugie rzeczywiste pociągnięcie narzędzia może zwiększyć ilość materiału.

Pierwszy etap: szczelna, niepokrywająca się wstęga z prawidłowymi caps/joins i
premultiplied alpha w liniowej przestrzeni. Samo przełączenie blend mode nie
naprawia podwójnego pokrycia z geometrii.

Drugi etap, jeśli próbki uzasadnią koszt: akumulacja umownej masy grafitu do osobnego
targetu, następnie kompozycja `R(M)`. Format, np. `R16Float`, wymaga sprawdzenia
obsługi render attachment/blending na docelowym device. Brak wymaganej funkcji
daje jawny raport lub świadomie wybrany profil jakości, nie cichy inny materiał.

Jeśli pozostają samoprzecięcia jednego gestu, ustalamy ich semantykę. Lokalne
pokrycie tego samego przejścia można połączyć przez maksimum przed depozycją;
rzeczywisty powrót narzędzia po tej samej linii może być kolejnym przejściem. Nie
tworzymy pełnowymiarowego targetu dla każdej kreski: rozważamy kafelki/batche albo
ograniczenie dopuszczalnych ścieżek. Każda metoda potrzebuje pomiaru kosztu.

Masa jest na początku statystyką materiału, nie pełną historią fizycznego papieru.
Rozcieranie, wycieranie gumką i zależny od kolejności burnishing wymagają później
bogatszego stanu. Nie obiecujemy ich dzięki samemu sumowaniu `deposit`.

## 12. Widoczność, clipping i passy

### 12.1. Osobny kontrakt powierzchni zasłaniającej

Obecne `fills` służą także do depth passu. Docelowy packet rozdziela:

```rust
struct NprRenderPacket {
    occluders: OcclusionGeometry,
    fills: Vec<DeclaredFill>,       // mogą być puste w rysunku ołówkiem
    strokes: IndexedStrokeBatches,
    materials: Vec<StrokeMaterial>,
    annotations: Vec<Annotation>,   // nie są geometrią modelu
    diagnostics: NprDiagnostics,
}
```

Znacznik wyboru z `mark_selection` nie powinien pozostawać parą fill triangles
z głębokością zero. Ma być jawną adnotacją z zadeklarowaną polityką głębokości,
rysowaną po materiale, niezanieczyszczającą testów tonu i zasłaniania.

### 12.2. Proponowana kolejność wewnątrz istniejącego World

```text
zebranie packetów i rezerwacja wspólnego budżetu
    -> depth wszystkich NPR occluders: jeden clear, Depth32Float, LessEqual, write
    -> papier/underlay zadeklarowanego widoku
    -> opcjonalny fill profilu: depth test, bez depth write
    -> widoczne ślady: depth test, bez depth write
    -> kompozycja materiału, jeśli użyto osobnego targetu depozycji
    -> adnotacje i zadeklarowane debug overlays
```

Nie czyścimy depth między obiektami. Warstwy grafitu i tuszu mają zadeklarowany
porządek kompozycji. Pipeline graficzny dostaje materiał i rolę jako dane; nie
wnioskuje, czy coś jest ołówkiem, na podstawie nazwy presetu/debug stringa.

To rozbudowa wewnętrznych passów w World, bez automatycznej przebudowy FrameGraph.
Wspólne zasłanianie NPR i zwykłych `MeshDrawCommand` wymaga osobnego audytu depth
i jawnego wkładu tych obiektów. Nie deklarujemy tej integracji jako otrzymanej
automatycznie przez zmianę packetu NPR. Zwykły renderer zachowuje swoją ścieżkę.

### 12.3. Poprawna interpolacja i clip

CPU może dalej podawać rzutowane pozycje. Wtedy trzeba pamiętać:

- Obecny znormalizowany depth `1 - near/z` może być liniowo interpolowany w ekranie.
  Nie jest to samo w sobie błąd perspektywy.
- Pozycja świata, normalne i parametry powierzchni potrzebują interpolacji
  perspektywicznej: `a = sum(lambda_i * a_i/w_i) / sum(lambda_i/w_i)`.
- Alternatywą jest zachowanie pozycji clip-space z właściwym `w` dla rasteryzera.
- Clipping tworzy nowe punkty z poprawnymi atrybutami, ale zachowuje oryginalną
  współrzędną ścieżki. Przycięcie oknem nie jest nowym gestem z nowym taperem.
- Near-plane i granice viewportu wymagają testów z liniami przecinającymi kamerę,
  zerową długością i bardzo dużymi współrzędnymi.

Przestrzenny overstroke nie może ujawniać linii z tylnej ściany. Wybrana polityka
konturu może pozwolić widocznej linii wyjść nieco poza sylwetkę, ale nie przez
bliższy obiekt. Mały depth bias łagodzi z-fighting; duży nie zastępuje widoczności.
Docelowo testujemy bias względem skali/depth slope, nie jedną magiczną stałą dla
każdej odległości.

## 13. Stabilność temporalna i płynny zoom

### 13.1. Determinizm nie jest temporal coherence

```text
deterministyczność: takie samo wejście i seed -> taki sam wynik
ciągłość: niewielka zmiana wejścia -> kontrolowana niewielka zmiana obrazu
```

Kontur zmienia zestaw krawędzi przy obrocie. Hash całego łańcucha może wtedy zmienić
wszystkie jego odchylenia. Potrzebujemy dopasowania fragmentów oraz przenoszenia
parametryzacji między kolejnymi widokami. Powierzchniowe hatchy są łatwiejsze:
mają trwałe kotwice, chociaż ich aktywność LOD nadal się zmienia.

Inspiracją dla oddzielenia linii geometrycznej od śledzonego śladu jest
[Active Strokes](https://gfx.cs.princeton.edu/pubs/Benard_2012_ASC/Benard_2012_ASC.pdf).
Planowany mechanizm dopasowania nie jest deklaracją implementacji ich pełnego
algorytmu aktywnych konturów.

### 13.2. Jawna historia domenowa

```rust
fn build_reference_frame(input: FrozenDrawingInput) -> NprRenderPacket;

fn advance_drawing(
    history: &mut DrawingHistory,
    input: DrawingFrameInput,
    dt: Seconds,
) -> NprRenderPacket;
```

Pierwsza funkcja służy testom i zrzutom z zamrożonym stanem. Druga ma jawny stan
sesji. Historia nie może ukrywać się w statycznym cache tessellatora ani w globalnym
RNG backendu. Implementacja należy do domeny NPR; plugin przechowuje instancję
i steruje jej cyklem życia.

Przykładowe dopasowanie:

```text
poprzednie kotwice -> rzutuj do nowego widoku
    -> kandydaci tej samej instancji/roli w lokalnym obszarze
    -> koszt: odległość + różnica stycznej + zgodność powierzchni + zakres ścieżki
    -> zaakceptuj tylko zgodne dopasowania pod progiem
    -> zachowaj ID/parametr; przy split/merge zachowaj pochodzenie fragmentów
    -> nowe ślady narodź; niepasujące wygaś
```

Nie wystarcza najbliższy punkt 2D: dwie różne powierzchnie mogą nakładać się w obrazie.
Na split/merge żaden globalny ID nie rozwiąże wszystkiego; przenosimy lokalne
przedziały parametru i pochodzenie. Dopasowanie ma ograniczoną liczbę kandydatów,
np. przez indeks przestrzenny, nie porównuje każdej linii z każdą.

### 13.3. Fading i histereza

Histereza LOD: osobny próg wejścia i wyjścia. Łagodzenie w czasie można opisać:

```text
alpha = 1 - exp(-dt / tau)
visibility_weight += (target_weight - visibility_weight) * alpha
```

`dt` to czas, nie numer klatki. Stabilność porównujemy przy 30/60/144 Hz w tych
samych chwilach symulacji. Waga artystyczna może wygasać łagodnie, ale zasłonięta
linia nadal podlega aktualnemu depth testowi. Nie zostawiamy duchów na obiekcie,
który przesunął się przed kreskę.

Zmiana kamery skokiem, sceny, geometrii, seeda i profilu ma jawne reguły resetu.
Resize aktualizuje projekcję i zasoby zależne od rozmiaru; nie musi zerować wszystkich
ID powierzchniowych. Zmiana target format wymaga wariantu pipeline'u, a zwykły resize
nie powinien go niepotrzebnie kompilować.

### 13.4. Trzy różne tryby czasu

- **Stabilny rysunek modelu w ruchu**: domyślny; zachowanie kotwic i ciągłości.
- **Animacja powstawania rysunku**: odtwarzanie kolejności gestów w czasie.
- **Celowo przerysowywana animacja**: opcjonalny styl z ograniczoną częstotliwością
  aktualizacji wariantu śladu; nigdy przypadkowy szum zależny od FPS.

Nie mieszamy trybu trzeciego z błędem migotania w trybie pierwszym.

## 14. Budżet, tessellacja i cache

### 14.1. Budżet przed alokacją, wspólny dla widoku

```rust
struct QualityBudget {
    max_candidate_steps: usize,
    max_visible_strokes: usize,
    max_vertices: usize,
    max_indices: usize,
    max_upload_bytes: usize,
    max_resident_bytes: usize,
    max_history_entries: usize,
}
```

Planner najpierw rezerwuje koszt ważnych konturów, potem głównego tonu, na końcu
poprawek i mikrodetalu. Rezerwacje rozdziela między widoczne obiekty/regiony według
jawnych priorytetów i pokrycia obrazu. Kolejność trójkątów nie może decydować,
który obiekt pozostanie niedorysowany.

Budżety jakości są deterministyczne. Zatrzymanie po arbitralnym czasie CPU daje
inny rysunek zależny od obciążenia komputera; jeżeli kiedyś dodamy adaptive quality,
musi być oddzielnym, jawnym trybem, wyłączanym w goldenach.

```text
bytes = vertices * size_of::<GpuStrokeVertex>()
      + indices  * size_of::<Index>()
      + material_buffers + frame_targets + bounded_history
```

W obliczeniach używamy checked arithmetic. Każdy pojedynczy bufor respektuje
`device.limits()`, a suma ma osobny limit. Uwzględniamy staging, stare pojemności
cache i klatki w locie. Nie czekamy, aż `create_buffer` zakończy aplikację.

Przekroczenie budżetu zgłaszamy przez diagnostykę: co odrzucono i dlaczego. Priorytet
„zachowaj kontury, redukuj detal” jest częścią kontraktu jakości, nie ukrytą
heurystyką backendu. Dla zbyt dużej geometrii bazowej zwracamy jawny błąd lub
wybieramy dostarczony LOD; nie usuwamy losowych trójkątów occludera.

### 14.2. Tańsza i poprawniejsza tessellacja

Docelowo indeksowana wstęga z dzielonymi wierzchołkami tam, gdzie atrybuty na to
pozwalają. Joiny tylko w miejscach zmiany kierunku; caps tylko na prawdziwych
końcach. Adaptacyjna liczba próbek zależy od błędu ekranowego, zmian szerokości
i głębokości, z twardym limitem kroków.

Nie wolno uprościć punktu tylko dlatego, że jest współliniowy w 2D: może nieść
istotną zmianę depth i położyć kreskę na niewłaściwej stronie powierzchni. Test
błędu obejmuje atrybuty potrzebne do widoczności i materiału.

WGPU wykonuje `draw_indexed` zamiast rozwijać każdy indeks do pełnego wierzchołka.
Chunks dzielimy z poprawnym remapem indeksów i kompletnością prymitywów. Reużywamy
bufory o kontrolowanej pojemności, ograniczamy szczytowy upload i liczbę klatek
utrzymujących stare bufory. Porcjowanie nie jest zgodą na nieograniczoną sumę.

### 14.3. Inwalidacja etapów

| Zmiana | Co przeliczyć | Co zachować |
| --- | --- | --- |
| Mesh / smoothing / granice | PreparedSurface, pola, kandydaci powierzchniowi | Niezależne zasoby innych meshy |
| Transform sztywny obiektu | Analiza widoku, projekcja, widoczność, ton zależny od światła | Topologia i pola w obiekcie |
| Skala niejednorodna | Normalne i metrykę odpowiednio przetransformować; przeliczyć zależną krzywiznę | Tylko cache, którego założenia nadal obowiązują |
| Kamera / zoom | Widok, selekcja LOD, projekcja i tessellacja | Kotwice i statyczne przygotowanie powierzchni |
| Światło | Pole tonu, aktywność hatchy, kontrastowe akcenty | Topologia, zasadnicze ścieżki powierzchniowe |
| Nacisk / szerokość | Odpowiedź narzędzia, kalibracja tonu, geometria śladu jeśli potrzebna | Geometria modelu i pole kierunków |
| Kolor papieru | Materiał/kompozycja; kalibracja tonu jeśli zmienia zakres | Topologia i kotwice |
| Ziarno / twardość | Materiał, kalibracja i ewentualne planowanie ilości śladów | Przygotowana geometria |
| Debug view | Zadeklarowany wariant wizualizacji i potrzebne dane | Dane niezależne od wybranego podglądu |
| Resize / DPI | Projekcja, LOD, tessellacja, targety rozmiaru | Pipeline'y o niezmienionym kluczu i stabilne kotwice |

Klucze cache uwzględniają rewizje danych, wersję algorytmu, seed i parametry naprawdę
używane przez etap. Nie porównujemy całego `Settings`, gdy zmieniła się wyłącznie
nazwa presetu w panelu. Transform wpływający na światło nie może z kolei zostawić
starego pola tonu przez zbyt agresywny cache.

Immutable snapshot powinien współdzielić duże przygotowane dane i gotowe batche
przez `Arc` tam, gdzie kontrakt na to pozwala. Obecne kopiowanie komend nie powinno
mnożyć rozmiaru całej tessellacji w serwisie, bridge i backendzie. Współdzielenie
nie zwalnia z kontroli liczby starych snapshotów w locie.

## 15. Cała klatka: pseudokod przepływu

```rust
// Update/PostUpdate: stan sceny i istniejący kontroler kamery.
apply_validated_control_commands(&mut scene_state);
update_animation(&mut scene_state, dt);
update_existing_camera_controller(&mut camera_state, dt);

// RenderExtract: raz na faktyczny snapshot widoku.
let inputs = snapshot_scene_inputs(scene_state, camera_state, actual_viewport);
let mut budget = ViewDrawingBudget::new(inputs.quality, backend_capabilities);

let prepared = inputs.instances.map(|instance| {
    surface_cache.get_or_prepare(instance.mesh_revision, instance.surface_policy)
}).collect();

let analysis = npr_domain.analyze_view(&prepared, &inputs);
let candidates = npr_domain.plan_candidates(&prepared, &analysis, &inputs.profile);
let selected = npr_domain.allocate_view_budget(candidates, &mut budget);

let paths = npr_domain.update_paths(&mut drawing_history, selected, inputs.time);
let gestures = npr_domain.realize_gestures(paths, &inputs.profile);
let batches = npr_domain.tessellate_bounded(gestures, &mut budget);

let packet = npr_domain.finish_packet(analysis.occluders, batches, budget.report());
render_service.publish_immutable(packet);

// Istniejący bridge kopiuje lekki uchwyt snapshotu, nie powtarza powyższej pracy.
extractor_bridge.contribute(render_service.snapshot(), &mut backend_frame_packet);

// World: wykonanie jawnych poleceń, bez decyzji o znaczeniu kresek.
backend.validate_resource_requirements(&backend_frame_packet)?;
backend.upload_bounded_batches(&backend_frame_packet)?;
backend.execute_declared_npr_passes(&backend_frame_packet)?;
```

`backend_capabilities` zawiera neutralne ograniczenia wymagane do planowania,
nie uchwyt WGPU w domenie. Domena wybiera jakość zgodnie z jawną polityką i limitami;
backend dodatkowo sprawdza poprawność packetu na granicy zaufania.

`analyze_view` nie oznacza obowiązkowego synchronicznego readbacku GPU co klatkę.
Pierwsza wersja korzysta z przygotowanych danych CPU i konserwatywnych zapytań;
ostateczne zasłanianie rozstrzyga depth test. Przyspieszenia zależne od danych GPU
wymagają osobnego projektu asynchronicznego wykonania i testu opóźnienia.

Obecny RenderExtract pozostaje miejscem publikacji, nie drugim Update. Nie
integrujemy ruchu kamery drugi raz podczas renderowania offscreen. Snapshot do
goldena powinien mieć ustaloną kamerę, orientację, czas i stan temporalny.

## 16. Warsztat UI i metadane Rhai

Wykorzystujemy istniejące [runtime scene panels](runtime-panels.md). Panel pozostaje
funkcją silnika, a dane i operacje przychodzą przez `RuntimeControlProvider`.
Egui nie dostaje bezpośredniego dostępu do structów NPR. Taki sam kontrakt będzie
mógł obsłużyć przyszły edytor.

### 16.1. Zakładki i kolejność pracy

| Zakładka | Zawartość podstawowa | Zawartość zaawansowana |
| --- | --- | --- |
| Galeria | Miniatury, nazwa modelu, prev/next, wybór, obrót, izolacja obiektu | Układ galerii, widoczność instancji |
| Rysunek | Profil, zakres zmian: scena/obiekt, kontrast, ilość detalu, porównanie A/B | Krzywa waloru, priorytety ról, lost edges |
| Narzędzie | Ołówek/tusz, rozmiar, nacisk, twardość lub kąt stalówki | Gest, długości końcówek, osobne poprawki |
| Kreskowanie | Gęstość, długość, prowadzenie po formie, druga warstwa | Pole kierunków, LOD, progi i histereza |
| Papier i światło | Papier, ziarno, kierunek światła, jasność | Parametry kontaktu, podkład, cienie |
| Diagnostyka | Tryb podglądu, czas CPU/GPU, liczniki, wykorzystanie budżetu | Przebiegi temporalne, cache, porównania testowe |

To proponowane uporządkowanie istniejącego warsztatu, nie sześć nowych okien.
Presety i kamera mogą pozostać dostępne jako stałe sekcje lub obecne zakładki,
jeżeli przebudowa nawigacji okaże się zbędna. Decyzję UX podejmujemy po obejrzeniu
aktualnego panelu; projekt nie zakłada przepisania go od zera.

Na start widoczne są kontrolki o dużym efekcie: profil, kontrast, rozmiar/nacisk,
ilość kreskowania, kierunek po formie, papier. Zaawansowane parametry rozwijają się
wewnątrz sekcji. Nie prezentujemy użytkownikowi od razu wszystkich parametrów szumu.

Przycisk „pokaż wpływ” izoluje daną warstwę w debug view. Tooltip wyjaśnia skutek
wizualny i jednostki, np. „dłuższe ślady opisują większe łuki formy”, nie tylko
powtarza nazwę `gesture_confidence`.

### 16.2. Czas: trzy niezależne ustawienia

- Limit renderowania/FPS, jeśli jest wystawiony przez właściwy serwis silnika.
- Prędkość animacji obiektu i pauza — stan sceny.
- Tryb/częstotliwość celowego przerysowywania śladów — polityka NPR.

Nie zmieniamy tempa obrotu przez liczbę klatek. Odświeżanie snapshotów panelu,
które ma własne ograniczenie, nie może stać się zegarem symulacji. Wygładzony zoom
korzysta z obecnego `zoom.rs`; poprawa rysunku nie wymaga jego zastąpienia.

### 16.3. Wiązania i atomowe operacje

Obecny provider korzysta z prefiksu `world.npr.settings.NprSettings.` i m.in.
aliasów `appearance.*`. Nowe nazwy poniżej są propozycją schematu, nie opisem
już działających bindings:

```text
appearance.profile
appearance.tone.contrast
appearance.lines.lost_edges
appearance.gesture.confidence
appearance.tool.pressure
appearance.hatching.form_alignment
appearance.paper.tooth
diagnostics.npr.upload_bytes          // tylko odczyt
diagnostics.npr.budget_limited        // tylko odczyt
```

Przykładowa intencja Rhai po zarejestrowaniu właściwej ścieżki:

```rhai
world.controls.set(
    "world.npr.settings.NprSettings.appearance.tool.pressure",
    0.55
);
```

Provider waliduje typ, zakres, dostępność narzędzia i komplet zależnych parametrów.
UI i Rhai mają identyczne reguły. Zmiana całego profilu/presetu jest atomowa:
nie publikujemy jednej klatki z nowym narzędziem i starym niezgodnym materiałem.

Przy zmianie schematu aktualizujemy razem provider, layout, skrypty, presety
w repozytorium i testy. Starszy nieobsługiwany zapis użytkownika dostaje jawny
komunikat z wersją; nie reinterpretujemy go po cichu. Nie nadpisujemy jego pliku
automatycznie. Ewentualny jednorazowy importer jest osobną, świadomą operacją.

Zmiany geometrycznie drogie łączymy do jednej aktualizacji snapshotu, pokazując
stan oczekujący. Parametry czysto materiałowe mogą reagować od razu. A/B zapisuje
kamerę, seed, czas i profil, aby porównanie dotyczyło wyglądu, nie innego ujęcia.

Istniejący IPC panelu ma ograniczenia wielkości i kolejki. Nie wysyłamy nim całych
packetów tessellacji ani pełnych targetów diagnostycznych. Liczniki są małe;
podglądy obrazu wymagają jawnego, ograniczonego kanału/zasobu, jeśli ich jeszcze
brakuje. Nie dodajemy specyficznego kodu NPR do generycznego `panel-egui`.

## 17. Mapa operacji w repozytorium

Wszystkie operacje poniżej są **planowane**, nie wykonane w ramach tego dokumentu.
Najpierw READ wskazanych symboli z aktualnego worktree; line numbers mogą się
zmienić. Nowe pliki powstają dopiero razem z używającym ich etapem, bez pustych
szkieletów na zapas. Pseudokod z rozdziałów 3–15 opisuje intencję implementacji.

Komendy są podane przez `rtk`, zgodnie z instrukcjami repozytorium. Testy o nowych
nazwach są proponowane; samo uruchomienie filtra z zerem testów nie jest walidacją.

### 17.1. Domena NPR

Wspólna granica tych operacji: bez zależności od WGPU, app i egui. Nie zmieniać
zachowania profilu komiksowego bez jawnej decyzji i przeglądu jego goldenów.

| Operacja i dokładna ścieżka | Symbol / zamiar | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `crates/engine/render-npr/src/style.rs` | Zastąpić rolę uniwersalnego worka `ComicInk` przez `DrawingProfile` i typed konstruktory profili | `rtk cargo check -p amigo-render-npr`; test walidacji profili | Wyboru profilu przez string w shaderze nie dodawać |
| MODIFY `crates/engine/render-npr/src/geometry.rs` | Uzupełnić dane potrzebne przygotowaniu; poprawnie nazwana gładka geometria testowa | `rtk cargo test -p amigo-render-npr geometry` | Canonical cube i jego świadomych ostrych krawędzi |
| MODIFY `crates/engine/render-npr/src/topology.rs` | `build_topology`: jawna walidacja, granice, winding, non-manifold | `rtk cargo test -p amigo-render-npr topology` | Nie scalać przypadkowo oddzielnych powierzchni |
| ADD `crates/engine/render-npr/src/surface.rs` | `NprPreparedSurface`, regiony gładkości, stabilne kotwice i rewizje | `rtk cargo test -p amigo-render-npr surface` | Nie trzymać tutaj bieżącego viewportu |
| ADD `crates/engine/render-npr/src/field.rs` | Kierunki styczne, transport, wiarygodność; później krzywizna | `rtk cargo test -p amigo-render-npr field` | Nie zgadywać polityki z nazwy modelu |
| ADD `crates/engine/render-npr/src/tone.rs` | Pole tonu, krzywe waloru i referencyjna kalibracja pokrycia | `rtk cargo test -p amigo-render-npr tone` | Nie narzucać liczby pasm wszystkim profilom |
| ADD `crates/engine/render-npr/src/visibility.rs` | W etapie cieni: domenowe zapytania CPU, struktury przyspieszenia, jawne błędy | `rtk cargo test -p amigo-render-npr visibility` | Nie tworzyć synchronicznego readbacku WGPU |
| MODIFY `crates/engine/render-npr/src/feature.rs` | Zachować trzy klasy; dopiero później dodać zweryfikowane linie gładkiej formy | `rtk cargo test -p amigo-render-npr feature` | Nie klasyfikować hatchy jako crease |
| MODIFY `crates/engine/render-npr/src/stroke.rs` | `StrokeRole`, stabilna ścieżka, pochodzenie i przedziały parametrów | `rtk cargo test -p amigo-render-npr stroke` | Nie traktować ID całego łańcucha jako pełnej historii |
| ADD `crates/engine/render-npr/src/hatching.rs` | Przenieść i zastąpić odpowiedzialność `append_hatching`: ścieżki powierzchniowe, progresywna gęstość | `rtk cargo test -p amigo-render-npr hatching` | Nie pozostawić starego generatora jako cichego fallbacku |
| MODIFY `crates/engine/render-npr/src/gesture.rs` | `sample` i `simplify`: korelowane procesy, trwały parametr, kontrola błędu atrybutów | `rtk cargo test -p amigo-render-npr gesture` | Nie losować po numerze wierzchołka/klatki |
| MODIFY `crates/engine/render-npr/src/tool.rs` | `ToolResponse`: jedno wyliczenie kontaktu, szerokości i jawna miękkość | `rtk cargo test -p amigo-render-npr tool` | Nie dublować nib aspect w tessellatorze |
| ADD `crates/engine/render-npr/src/material.rs` | Neutralne parametry papieru, depozycji i profilu brzegowego | `rtk cargo test -p amigo-render-npr material` | Nie wprowadzać uchwytów tekstur WGPU |
| MODIFY `crates/engine/render-npr/src/tessellation.rs` | Indeksowana wstęga, adaptacyjne caps/joins, atrybuty materiału, limity | `rtk cargo test -p amigo-render-npr tessellation` | Nie usuwać ograniczeń alokacji bez zastępstwa |
| ADD `crates/engine/render-npr/src/temporal.rs` | `DrawingHistory`, dopasowanie, narodziny/wygaszanie, reset | `rtk cargo test -p amigo-render-npr temporal` | Nie ukrywać globalnego mutowalnego stanu |
| ADD `crates/engine/render-npr/src/budget.rs` | Rezerwacje kosztu dla całego widoku, priorytety i raport | `rtk cargo test -p amigo-render-npr budget` | Nie obcinać po kolejności ścian |
| MODIFY `crates/engine/render-npr/src/frame.rs` | `NprRenderPacket`, `build_packet_with_topology`: orkiestracja nowych etapów, osobne occluders/adnotacje | `rtk cargo check -p amigo-render-npr`; `rtk cargo test -p amigo-render-npr` | Nie przenosić algorytmów do pluginu |
| MODIFY `crates/engine/render-npr/src/camera.rs` | Zachowanie parametrów przy clip, jawne atrybuty perspektywiczne i viewport | `rtk cargo test -p amigo-render-npr projection` | Nie zmieniać konwencji depth przypadkiem |
| MODIFY `crates/engine/render-npr/src/debug.rs` | ToneTarget, StrokeRoles, FieldDirections, Pressure, Coverage, Temporal, Budget | `rtk cargo test -p amigo-render-npr debug` | Nie usuwać Final/FeatureClasses/StrokeIds |
| MODIFY `crates/engine/render-npr/src/lib.rs` | Eksportować rzeczywiście użyte nowe moduły i poprawić opis kontraktu | `rtk cargo check -p amigo-render-npr` | Nie eksportować backendowych detali |

Po przeniesieniu odpowiedzialności wykonujemy DELETE starej funkcji
`append_hatching` w `frame.rs` i zbędnych helperów, ale tylko gdy nowy generator
obsługuje zadeklarowane profile i testy. Weryfikacja: ukierunkowane `rtk rg -n
"append_hatching" crates/engine/render-npr/src`, następnie testy owner crate.

### 17.2. Kontrakty, GPU i bridge

| Operacja i dokładna ścieżka | Symbol / zamiar | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `crates/engine/render-api/src/npr.rs` | `NprDrawCommand`, `NprBackgroundCommand`: jawny materiał i lekkie snapshoty, bez heurystyk presetu | `rtk cargo check -p amigo-render-api` | Pozostałych draw commandów |
| MODIFY `crates/engine/render-api/src/stats.rs` | `RenderFrameStats`: koszt NPR, role, cache, ograniczenia jakości | `rtk cargo check -p amigo-render-api` | Znaczenia istniejących liczników innych domen |
| MODIFY `crates/engine/render-wgpu/src/frame_packet.rs` | Przechowywanie nowego snapshotu, poprawny reset zawartości | `rtk cargo check -p amigo-render-wgpu` | Pustego NPR jako pustego wkładu |
| MOVE `crates/engine/render-wgpu/src/renderer/npr.rs` -> `crates/engine/render-wgpu/src/renderer/npr/mod.rs` | Dopiero przy rozdzieleniu wykonania; jeden finalny moduł `npr` | `rtk cargo check -p amigo-render-wgpu` | Nie zostawiać równoległego `npr.rs` |
| ADD `crates/engine/render-wgpu/src/renderer/npr/buffers.rs` | Bufory indeksowane, jawne limity uploadu, reużycie pojemności | `rtk cargo test -p amigo-render-wgpu npr` | Nie tracić ochrony limitu pojedynczego bufora |
| ADD `crates/engine/render-wgpu/src/renderer/npr/passes.rs` | Wykonanie zadeklarowanego porządku depth/fill/material/composite | `rtk cargo test -p amigo-render-wgpu npr` | Nie czyścić depth per obiekt |
| ADD `crates/engine/render-wgpu/src/renderer/npr/shaders/stroke.wgsl` | Profil odległości, papier, materiał, poprawne pokrycie | `rtk cargo test -p amigo-render-wgpu npr` z faktycznym utworzeniem pipeline'u | Nie umieszczać selekcji kresek w shaderze |
| ADD `crates/engine/render-wgpu/src/renderer/npr/shaders/paper.wgsl` | Wspólne, filtrowane pole papieru | Jak wyżej plus swatch offscreen | Nie losować innego papieru per obiekt |
| ADD `crates/engine/render-wgpu/src/renderer/npr/shaders/composite.wgsl` | Dopiero dla targetu depozycji: materiał -> reflektancja | Testy offscreen tonu i mieszania | Nie mylić linear RGB z sRGB |
| MODIFY `crates/engine/render-wgpu/src/renderer/service/render/world.rs` | `render_npr_commands`: przekazać zbiorczy snapshot do passów, usunąć rozwijanie indeksów | `rtk cargo check -p amigo-render-wgpu`; testy NPR | Starej ścieżki `MeshDrawCommand` i innych passów |
| MODIFY `crates/engine/render-wgpu/src/renderer/service/model.rs` | Zasoby NPR i jawny cykl życia cache | `rtk cargo check -p amigo-render-wgpu` | Nie dodawać pól osobno dla każdego artystycznego presetu |
| MODIFY `crates/engine/render-wgpu/src/renderer/service/init.rs` | Tworzenie/rebuild zasobów zależnie od device/formatu/sample count | `rtk cargo check -p amigo-render-wgpu` | Nie przebudowywać pipeline'u tylko z powodu resize |
| MODIFY `crates/runtime/bundles/src/render_extractor_bridges/world_3d.rs` | Lekko przenieść jawny snapshot; zachować rejestrację extractora | `rtk cargo check -p amigo-runtime-bundles` po zielonym backendzie | Nie powtarzać projekcji, stylowania i tessellacji |

Zmiana współdzielonego kontraktu packetu może wymagać małych dostosowań obu dróg
raportowania: `crates/runtime/bundles/src/render_session.rs` oraz
`crates/apps/app/src/render_runtime.rs`. Operacja MODIFY dotyczy tylko przekazania
snapshotu/liczników; weryfikacja: check owner crate, potem
`rtk cargo check -p amigo-app`. Nie dodawać zależności ani algorytmów domenowych do
app. Jeśli powtarzane przypisywanie stats wymaga wspólnego helpera, jego właścicielem
jest kontrakt/warstwa renderowania, nie aplikacja.

### 17.3. Plugin, import, mod i panele

| Operacja i dokładna ścieżka | Symbol / zamiar | Walidacja | Nie zmieniać |
| --- | --- | --- | --- |
| MODIFY `plugins/gfx/npr-playground/src/render/mod.rs` | `Prepared`, `rebuild`, `commands`: używać domenowego przygotowania i współdzielonego snapshotu | `rtk cargo check -p amigo-npr-playground-plugin` | Nie implementować pól/gestów w pluginie |
| MODIFY `plugins/gfx/npr-playground/src/plugin.rs` | Jawna historia per sesja/widok, reset sceny, raz wykonywany RenderExtract | `rtk cargo test -p amigo-npr-playground-plugin` | Nie aktualizować kamery drugi raz |
| MODIFY `plugins/gfx/npr-playground/src/state.rs` | `Settings`, `ObjectSettings`, `RuntimeControlProvider`: typed profile, rewizje, walidacja i diagnostyka | Testy providera/presetów w crate pluginu | Nie pozwalać UI omijać walidacji |
| MODIFY `plugins/gfx/npr-playground/src/state/look_presets.rs` | Wersjonowany zapis kompletnego profilu i atomowy odczyt | `rtk cargo test -p amigo-npr-playground-plugin preset` | Nie nadpisywać niezgodnych plików użytkownika |
| MODIFY `plugins/gfx/npr-playground/src/state/history.rs` | Historia edycji obejmuje kompletny profil | Test undo/redo w crate pluginu | Nie mieszać historii edycji z historią temporalną rysunku |
| MODIFY `crates/3d/mesh/src/geometry_asset.rs` | Tylko gdy wymagane: zachowanie jawnych normalnych/szwów/identyfikatorów wejścia | `rtk cargo check -p amigo-3d-mesh`; testy importu | Nie rozszerzać importera bez wykazanego wymagania |
| MODIFY `mods/npr-playground/ui/npr.panel.yml` | Sekcje narzędzi, podstawowe kontrolki, bindings i diagnostyka | Test ładowania panelu/pluginu i sesja hosted | Nie umieszczać tu algorytmów |
| MODIFY `mods/npr-playground/scenes/gallery/scene.yml` | Zadeklarowane profile, modele referencyjne, panel auto_open | Test sceny i plugin-check | Nie dodawać drugiego mesha renderującego te same obiekty |
| ADD `mods/npr-playground/scenes/stroke-lab/scene.yml` | Plansza próbek w domenie NPR; authored wybór sceny | Test ładowania plus offscreen | Mod nie implementuje modelu narzędzia |
| MODIFY `plugins/gfx/npr-playground/README.md` | Instrukcja warsztatu i odnośnik do niniejszej architektury przy wdrożeniu | `rtk git diff --check` | Nie reklamować niewdrożonych etapów jako gotowych |
| MODIFY `plugins/gfx/npr-playground/docs/pipeline.md` | Rzeczywisty przepływ stanu i nowych snapshotów | `rtk git diff --check` | Nie duplikować całej teorii z tego dokumentu |
| MODIFY `plugins/gfx/npr-playground/docs/contributions.md` | Kontrakty extractora i profilu | `rtk git diff --check` | Nie zmieniać capabilities bez testów rejestracji |
| MODIFY `plugins/gfx/npr-playground/docs/diagnostics.md` | Znaczenie, jednostki i częstotliwość liczników | `rtk git diff --check` | Nie logować całego raportu każdej klatki release |
| MODIFY `plugins/gfx/npr-playground/tests/waterfall_tests.rs` | Pełny przepływ provider -> stan -> RenderExtract -> komendy/diagnostyka | `rtk cargo test -p amigo-npr-playground-plugin` | Nie zastępować testu wyniku samą obecnością serwisu |

`crates/ui/panel-api/src/lib.rs` i `crates/ui/panel-egui/src/lib.rs` pozostają READ,
chyba że rzeczywiście brakuje generycznego widgetu/kontraktu. Ewentualne MODIFY musi
być domenowo neutralne i przejść testy `amigo-panel-api`/`amigo-panel-egui`.
Nie dodajemy nowej aplikacji edytora ani drugiego protokołu komunikacji paneli.

## 18. Kolejność wdrażania i bramki jakości

### M0. Powtarzalna baza i kontrola kosztu

Zakres: ustalone ujęcia galerii, zapis pełnych statystyk klatki, instrumentacja
szczytowego kosztu, reprodukcja zgłoszonej alokacji, podstawowa walidacja packetu.
Dokładny root cause błędu pamięci zapisujemy dopiero po jego odtworzeniu.

Bramka: aplikacja przedstawia rzeczywistą klatkę i panel; błąd danych/limitu nie
kończy się niekontrolowaną paniką WGPU. Mamy liczby CPU/upload/resident bytes dla
zamrożonej sceny. Istnienie procesu i komunikat inicjalizacji GPU nie wystarczają.

### M1. Laboratorium śladu: pierwszy widoczny efekt

Zakres: `material.rs`, odpowiedź narzędzia, poprawna tessellacja, shader śladu,
papier współdzielony z materiałem, indeksowany upload i testowe swatche. Na tym
etapie nie potrzebujemy krzywizny ani modelu organicznego.

Plansza: prosta, łuk, S, ostre połączenie, kilka nacisków, pojedyncze/dwa przejścia,
różne tła i papiery. Jeden pewny gest powinien wyglądać jak ślad grafitu nawet bez
dużego odchylenia geometrycznego. Poprawiamy kontakt i brzeg, jeśli nadal wygląda
jak przezroczysta plastikowa wstążka.

Bramka: nacisk i papier mają czytelny, monotoniczny wpływ; brak ciemnych kropek na
joinach; poprawne końcówki; powtarzalny wynik przy różnej tessellacji; mniejszy
koszt uploadu niż rozwinięta lista tych samych indeksów.

### M2. Pierwszy rysunek bryły zamiast cel-shadingu

Zakres: osobne occluders, pole tonu, gładka kula/cylinder, jawne proste pole
kierunków, ścieżki przekraczające triangulację, jedna rodzina hatchy. Selektywny,
delikatny kontur. Kolorowy three-band fill wyłączony tylko w profilu `PencilStudy`.

Bramka: bryła czytelna w miniaturze bez kolorowego fill; hatchy opisują kształt;
nie widać wewnętrznych diagonali ani „szwów” z zakończeń kresek na trójkątach;
przysłonięte linie znikają. Zmiana triangulacji zachowuje podobny ton i kierunek,
choć nie musi dawać bitowo tych samych ID.

To pierwszy kamień milowy prezentowany jako „rysunek ołówkiem”. M1 bez M2 daje
wiarygodniejszy materiał kreski, ale sam nie zmienia logiki całej ilustracji.

### M3. Kontrolowany gest, walory i spójność w ruchu

Zakres: trwała parametryzacja, hierarchiczny szum, asymetria gestu, wybrane poprawki,
progresywna gęstość, podstawowy temporal matching i LOD/histereza. ID oraz parametry
potrzebne do tego etapu są przewidziane już w M1/M2.

Bramka: wolny obrót i zoom nie wymieniają całego rysunku; brak resetu taperu na
granicy okna; przerwy konturu są uzasadnione walorem/rolą; 30/60/144 Hz nie zmienia
istotnie zachowania w tych samych chwilach animacji.

### M4. Ogólne powierzchnie, pełniejsze światło i wiele warstw

Zakres: przygotowanie importowanych powierzchni, pola z krzywizny i ich confidence,
cross-hatching, cienie rzucane/kontaktowe na jawnych odbiornikach, lepsza kontrola
pokrycia. Opcjonalny target depozycji tylko po porównaniu z prostszym modelem.

Bramka: kula, cylinder i model organiczny zachowują własny charakter; niestabilne
krzywizny nie tworzą jeżowatego szumu; cień kontaktu nie jest zgadywany z pozycji
obiektu na ekranie; koszt galerii mieści się w zadeklarowanym globalnym budżecie.

### M5. Zaawansowane linie formy, tusz i wykończenie warsztatu

Zakres: suggestive contours po walidacji krzywizny, kolejne narzędzia, kalibrowane
presety, pełne A/B, debug views, testy między profilami. Rdzeń nie przyjmuje
nowych renderer-side heurystyk wraz z każdym presetem.

Bramka: profil tuszu ma własną czytelną charakterystykę kąta/przepływu, a ołówek
nie traci grafitowego medium. Przegląd wizualny potwierdza zamierzony styl, nie tylko
inne wartości suwaków. Dokumentacja opisuje to, co naprawdę działa.

### Kolejność priorytetów

Największą zmianę obrazu powinno dać połączenie M1 + M2. Nie zaczynamy od pełnej
symulacji włosia, gumki, setek parametrów motoryki ani suggestive contours na
nieprzygotowanej siatce. Te mechanizmy wymagają już czytelnego tonu i materiału,
aby można było rzetelnie ocenić ich wkład.

## 19. Weryfikacja: nie tylko „kompiluje się”

### 19.1. Testy domenowe

Istniejący `crates/engine/render-npr/tests/geometry_tests.rs` pozostaje właścicielem
regresji bazowej geometrii. Proponowane ADD, w miarę realizacji etapów:

- `crates/engine/render-npr/tests/drawing_material_tests.rs`: swatchowe dane
  wejściowe, nacisk, kontakt, monotoniczność krzywych, zero pokrycia i nasycenie.
- `crates/engine/render-npr/tests/surface_hatching_tests.rs`: przejścia między
  trójkątami, brak sztucznych zakończeń, pola kierunków, limity integracji.
- `crates/engine/render-npr/tests/temporal_tests.rs`: stabilne kotwice, split/merge,
  histereza, ciągłość po clip/resize i reset historii.
- `crates/engine/render-npr/tests/budget_tests.rs`: globalne rezerwacje, checked
  arithmetic, przewidywalna redukcja, wiele obiektów, limity przed alokacją.

Walidacja każdej operacji: najpierw `rtk cargo check -p amigo-render-npr`, potem
`rtk cargo test -p amigo-render-npr --test <nazwa_pliku_bez_rs>`. Nie przenosić tych
testów do app ani uzależniać matematyki od dostępności GPU.

Istotne własności do sprawdzenia:

| Obszar | Własność |
| --- | --- |
| Determinizm | To samo wejście, seed i jawna historia dają ten sam wynik w ustalonym środowisku |
| Próbkowanie | Zmiana liczby próbek odczytuje ten sam gest z tolerancją błędu, nie inny szum |
| Triangulacja | Inny podział tej samej płaszczyzny nie tworzy widocznych szwów ani innego średniego tonu |
| Topologia | Niefinitywne dane, błędne indeksy i non-manifold mają kontrolowany wynik |
| Pole | `d` i `-d` nie znoszą się podczas wygładzania; kula nie produkuje losowych eigenvectors |
| Gest | Nacisk i końce są ciągłe, długość clip fragmentu nie resetuje fazy |
| Narzędzie | Kąt stalówki zmienia szerokość zgodnie z jawnym modelem; nacisk nie działa podwójnie |
| Budżet | Przy wielu packetach suma respektuje limit widoku, z raportem odrzuconych ról |
| Widoczność | Uproszczenie 2D nie łamie depth; fragmenty poza near nie produkują ogromnych współrzędnych |
| Temporal | Zmiana FPS nie losuje nowych kresek; camera cut usuwa niezgodną historię |

Bitowej zgodności obliczeń zmiennoprzecinkowych między różnymi CPU/GPU nie
zakładamy bez osobnego dowodu. Dokładne testy ID/struktur rozdzielamy od testów
numerycznych i obrazowych z tolerancją.

### 19.2. Testy WGPU

Testy rzeczywiście tworzą pipeline'y WGSL i renderują próbki offscreen. Sam
`cargo check` nie dowodzi poprawności layoutów shaderów ani blendowania.

Proponowane ADD `crates/engine/render-wgpu/src/renderer/npr/tests.rs`, po MOVE
modułu: walidacja layoutów, indeksów/chunków, targetów i offscreen swatches.
Weryfikacja: `rtk cargo test -p amigo-render-wgpu npr`. Nie zmieniać generycznych
testów innych rendererów tylko w celu dopasowania nowego wyglądu.

Scenariusze: materiał na jasnym i ciemnym tle, join bez kropki, koniec z miękkim
brzegiem, dwa prawdziwe przejścia, obiekty wzajemnie zasłaniające, stroke wychodzący
poza ekran, target po resize i brak NPR. Sprawdzamy również rzeczywistą liczbę
wywołanych testów. Brak adaptera oznacza jawny brak części weryfikacji, nie „pass”.

### 19.3. Goldeny i porównania wizualne

Punktem integracji pozostaje
`crates/apps/app/src/tests/scene_loading_tests/threed.rs`,
`npr_playground_offscreen_matches_packet_contract`, oraz istniejący
`mods/npr-playground/tests/golden/cube-512.png`.

Planowane MODIFY testu/ADD sąsiednich testów: zachować kontrolę profilu komiksowego
i dodać osobne referencje ołówka. Nowe obrazy ADD dopiero po rzeczywistym wyrenderowaniu
i przeglądzie, np. `pencil-sphere-512.png` i `pencil-stroke-lab-512.png` w tym samym
katalogu goldenów. Nie generować „poprawnych” referencji przez automatyczne
zaakceptowanie dowolnego aktualnego outputu.

Warunki referencji:

- 512×512, ustalony viewport i skala strony.
- Dla cube zachować ustalone `rx=0.36`, `ry=0.71` i stały seed.
- Jawny profil, światło, model, kamera, rewizja algorytmu i tryb historii.
- Bez adnotacji wyboru/UI w obrazie materiału, chyba że dany test je sprawdza.
- Osobne statystyki dla geometrii, ról i faktycznych zasobów backendu.

Porównujemy obraz w kilku skalach: niska częstotliwość dla mas waloru, krawędzie
dla ciągłości formy, powiększone fragmenty dla materiału. Tolerancje ustalamy na
zaakceptowanych różnicach adapterów, a nie dowolnie podnosimy aż test przejdzie.
Można mierzyć błąd luminancji/gradientu i odsetek różniących się pikseli, ale żadna
pojedyncza metryka nie potwierdza „ludzkości” rysunku.

Walidacja po zielonych niższych warstwach:
`rtk cargo test -p amigo-app npr_playground_offscreen_matches_packet_contract`.
Nowe testy mają własne filtry. Aktualizacja goldenów wymaga świadomego przeglądu:
test zapisuje tylko kandydata przez `AMIGO_CAPTURE_NPR_GOLDEN_DIR`, a PNG oraz
fingerprint są zmieniane jawnie w tym samym review.

### 19.4. Sekwencje ruchu i wydajność

Zestaw: statyczny obraz, 36 ustalonych rotacji, dodatkowo gęsta sekwencja wolnego
obrotu, płynny zoom in/out, wejście w near plane, resize panoramiczny/pionowy,
minimalizacja/zerowy rozmiar, 512p/1080p/4K oraz wiele instancji. Same 36 klatek
mogą nie ujawnić krótkiego migotania, więc nie zastępują ciągłej sekwencji.

Przy ruchu porównanie obrazu uwzględnia ruch kotwic, widoczność i dopuszczalne
narodziny/zaniki śladów. Surowa różnica sąsiednich klatek myli prawidłowy obrót
z niestabilnością kreskowania.

Raport wydajności obejmuje przynajmniej:

```text
npr.prepare_ms / analyze_ms / hatch_ms / gesture_ms / tessellate_ms
npr.upload_bytes / resident_bytes / peak_cpu_bytes
npr.candidates / accepted_strokes / rejected_by_budget
npr.vertices / indices / draw_calls / buffer_chunks
npr.history_entries / births / deaths / unmatched
npr.cache_hits / cache_misses / viewport / profile / debug_view
```

Nazwy i jednostki ustalamy w istniejącym mechanizmie diagnostyk. Czas GPU raportujemy
tylko, gdy został naprawdę zmierzony; nie nazywamy czasu budowania komend czasem GPU.
Release publikuje liczniki/agregaty, nie pełny tekst raportu co klatkę.

Docelowy FPS, limit uploadu i pamięci ustalamy po M0 dla konkretnego sprzętu oraz
liczby modeli. Nie ma tutaj obietnicy 60 FPS na RTX 3070 Ti bez pomiaru. Jakość
interaktywna i jakość eksportu mogą mieć różne jawne budżety, ale ten sam model
rysunku i ten sam sposób interpretacji profilu.

### 19.5. Odbiór człowieka

Każdy milestone oglądamy w zamrożonym ujęciu i w ruchu. Pytania odbiorowe:

1. Czy po pomniejszeniu widać bryłę i grupy walorów, a nie tylko siatkę linii?
2. Czy kreski pomagają odczytać wypukłość i kierunek powierzchni?
3. Czy papier oddziałuje ze śladem, a nie jest wyłącznie brudnym tłem?
4. Czy kontur ma hierarchię bez utraty czytelności kształtu?
5. Czy zmiany szerokości/nacisku wyglądają jak gest konkretnego narzędzia?
6. Czy po wyłączeniu wobble obraz nadal ma charakter ołówka?
7. Czy obraz nie miga, nie pływa i nie odsłania tylnych kresek podczas zoomu?
8. Czy większa ilość detalu rzeczywiście poprawia rysunek?

Akceptacja interaktywna obejmuje uruchomienie
`rtk cargo run -p amigo-app -- --mod npr-playground --scene gallery`, widoczne
okno renderowania, działający panel i zmianę ustawień potwierdzoną w obrazie.
`stroke-lab` uruchamiamy analogicznie dopiero po dodaniu tej sceny.

Po zmianach integracyjnych pluginu uruchamiamy również
`rtk cargo run -p amigo-plugin-check -- validate plugins/gfx/npr-playground`.
Walidacja manifestu i dokumentacji nie zastępuje testu działającego okna. Zakres
sprawdzeń rozszerzamy stopniowo od owner crate; nie uruchamiamy domyślnie całego
workspace ani formatowania niezwiązanych plików.

## 20. Ryzyka i świadome granice

- **Geometria nie zawiera całej intencji artysty.** W dłuższej perspektywie potrzebne
  mogą być autorskie regiony, kierunki, linie i mapy ważności.
- **Im bardziej stylizowany ślad, tym trudniejsza jego widoczność.** Duże odchylenie
  od źródłowej powierzchni musi mieć jawne zasady, szczególnie przy sylwetce.
- **Gęsty grafit nie musi być milionem krzywych.** Dla dalszych planów można
  rozważyć inny zadeklarowany reprezentant tonu, np. filtrowany atlas hatchingu.
  To osobny tryb reprezentacji z testem zgodności, nie automatyczny fallback WGPU.
- **Temporality bywa historią zależną od ścieżki kamery.** Referencja statyczna
  i odtwarzanie sekwencji są oddzielnymi przypadkami testowymi.
- **Budżet może zmienić wygląd.** Wymaga widocznej diagnostyki i kontrolowanej
  redukcji, a nie cichego ucinania ostatnich obiektów galerii.
- **Preset to nie fizyczna certyfikacja medium.** Realizm oceniamy na próbkach,
  z jasno określoną skalą i oświetleniem.
- **Licencje dotyczą też danych.** Kod referencyjny i modele/papiery/próbki mogą
  mieć inne warunki użycia niż opis algorytmu. Przykładowo projekt `rtsc` deklaruje
  GPL; plan zakłada własną implementację metod, nie bezrefleksyjne kopiowanie kodu.
  Każde przejęcie kodu lub assetu wymaga osobnego sprawdzenia warunków.

Poza pierwszym zakresem: pełna symulacja włosia, fizyczny chwyt ręki, zużywanie
grafitu, gumka i rozcieranie z historią materiału, rysowanie po animowanej deformacji
skóry, trenowanie modeli stylu na cudzych rysunkach, nowy edytor i przebudowa całego
FrameGraph. Można je projektować później bez blokowania pierwszego dobrego rysunku.

## 21. Referencje i jak je wykorzystać

To źródła metod i porównań, nie gotowa specyfikacja Amigo. Projektowane typy,
budżety, etapy i integracja są propozycją dostosowaną do tego repozytorium.

1. Winkenbach, Salesin, 1994 — [Computer-Generated Pen-and-Ink Illustration](https://grail.cs.washington.edu/projects/cg-illus/).
   Kontekst budowania ilustracji scen 3D za pomocą śladów tuszu.
2. Praun, Hoppe, Webb, Finkelstein, 2001 — [Real-Time Hatching](https://gfx.cs.princeton.edu/proj/hatching/).
   Tonal art maps, spójność skali/tonu; punkt odniesienia dla progresywnego detalu.
3. Webb i in., 2002 — [Fine Tone Control in Hardware Hatching](https://gfx.cs.princeton.edu/pubs/Webb_2002_FTC/index.php).
   Punkt odniesienia dla rozdzielenia kontroli tonu od wzoru śladów.
4. DeCarlo i in., 2003 — [Suggestive Contours for Conveying Shape](https://gfx.cs.princeton.edu/pubs/DeCarlo_2003_SCF/DeCarlo2003.pdf).
   Matematyczne warunki dodatkowych linii gładkiej formy.
5. DeCarlo, Finkelstein, Rusinkiewicz, 2004 — [Interactive Rendering of Suggestive Contours with Temporal Coherence](https://gfx.cs.princeton.edu/pubs/DeCarlo_2004_IRO/index.php).
   Stabilność tego rodzaju linii w ruchomym widoku.
6. Rusinkiewicz, 2004 — *Estimating Curvatures and Their Derivatives on Triangle Meshes*,
   wskazane w [bibliografii projektu Suggestive Contours](https://gfx.cs.princeton.edu/proj/sugcon/index.html).
   Estymacja krzywizny i pochodnych; ważne przed wdrożeniem linii wyższych rzędów.
7. Cole i in., 2008 — [Where Do People Draw Lines?](https://gfx.cs.princeton.edu/proj/ld3d/lineset/index.html).
   Dane i badanie wyboru linii przez artystów. Przydatne do oceny, czy selekcja
   geometryczna rzeczywiście wspiera czytelność.
8. Sousa, Buchanan, 2000 — [Observational Models of Graphite Pencil Materials](https://onlinelibrary.wiley.com/doi/abs/10.1111/1467-8659.00386).
   Modelowanie medium i porównania próbek; punkt odniesienia dla laboratorium śladu.
9. Bénard i in., 2012 — [Active Strokes: Coherent Line Stylization for Animated 3D Models](https://gfx.cs.princeton.edu/pubs/Benard_2012_ASC/Benard_2012_ASC.pdf).
   Oddzielenie śledzonych śladów od zmieniających się próbek cech geometrycznych.
10. Flash, Hogan, 1985 — [The coordination of arm movements: an experimentally confirmed mathematical model](https://pubmed.ncbi.nlm.nih.gov/4020415/).
    Inspiracja dla gładkiego przebiegu gestu, z ograniczeniami opisanymi w rozdziale 9.

Praktyczna kolejność czytania do wdrożenia: medium i hatching dla M1/M2, wybór
linii dla hierarchii, Active Strokes dla M3, krzywizna i suggestive contours przed
M4/M5. Nie trzeba zaimplementować wszystkich publikacji, żeby osiągnąć pierwszy
wiarygodny efekt.

## 22. Pierwsze konkretne zadanie implementacyjne

**M0 + początek M1: powtarzalny stroke lab, poprawna indeksowana wstęga i shader
grafitu korzystający ze wspólnego papieru.**

Gotowy rezultat tego zadania powinien zawierać planszę pojedynczych śladów,
zmienny nacisk i końce, brak artefaktów na joinach, bezpieczne bufory, dane o koszcie
i zaakceptowany obraz referencyjny. Następne zadanie to M2: zachować depth-only
bryłę i zastąpić kolorowe pasma polem tonu realizowanym przez hatchy po formie.

To najmniejsza sekwencja, która pozwala osobno ocenić wiarygodność materiału
i wiarygodność całego rysunku, z czytelnym powiązaniem każdego efektu z kodem.
