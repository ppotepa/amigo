# codemap-tests summary

Podsumowanie po przejściu `001`, `002`, `003`, `004`.

Uwaga:
- `001-003` były read-only,
- `004` był wykonany z realną implementacją i świadomym rollbackiem po obu metodach,
- tokeny są szacowane z długości outputu komend (`chars / 4`), więc to metryka porównawcza, nie billing API.

## Totals

- Used total (codemap-first): ~6289
- Used total (standard-first): ~28276
- Saved total: ~21987
- Output lines total:
  - codemap-first: 1333
  - standard-first: 3479
- Manual files opened total:
  - codemap-first: 2
  - standard-first: 12

## Best reports

- Highest signal: `impact`, `orphan-files`, `large-files`
- Best for navigation: `open-set`, `slice`
- Best for cleanup: `orphan-files`, `delete-plan`, `stale`
- Best for verification: `verify-plan`, `import-fix-plan`, `fallout`

## Weak spots

- False positives:
  - `open-set` nadal potrafi pokazać trochę szumu z narzędzi/docs w `skip`, choć nie wpuszcza ich już do listy głównej
  - `asset-file-check` pozostaje heurystyczny domenowo mimo parsera YAML
- Missing heuristics:
  - mocniejsze area grouping dla niektórych dużych plików frontendowych
  - jeszcze lepsze candidate ranking dla moved imports
- Cases where manual `rg`/`git diff` was still necessary:
  - potwierdzenie dokładnych callsite'ów przy symbol migration
  - szybki sanity check przy cleanup candidate
  - pełny kontekst dużego pliku, jeśli split ma być faktycznie wdrażany, nie tylko planowany

## Per-task notes

### 001 Symbol Migration
- Used: codemap ~940 / standard ~3698
- Saved: ~2758
- Best command: `impact`
- Notes:
  - najlepszy stosunek sygnału do kosztu
  - dobrze wskazał warstwy zależne od `EditorSelectionRef`

### 002 File Ops Cleanup
- Used: codemap ~1007 / standard ~7331
- Saved: ~6324
- Best command: `orphan-files`
- Notes:
  - największa oszczędność względem ręcznego otwierania plików sąsiednich
  - dobrze działa dla shim/empty cleanupu

### 003 Large File Split
- Used: codemap ~1542 / standard ~9647
- Saved: ~8105
- Best command: `large-files` + `move-plan`
- Notes:
  - największy zysk przy unikaniu pełnego czytania dużego pliku
  - dobre do planowania, przed właściwym refaktorem i build fallout

### 004 Project Explorer Model
- Used: codemap ~2800 / standard ~7600
- Saved: ~4800
- Best command: `open-set` + `slice`
- Notes:
  - dobry przykład dla prawdziwej implementacji, nie tylko nawigacji
  - `open-set` dobrze zawęził pliki do ekstrakcji modelu
  - standardowy flow szybciej rozlał się na pliki niskiej wartości
