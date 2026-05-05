# codemap-tests

Scenariusze ręcznych testów operacyjnych dla `amigo-codemap`.

Cel:
- przejść 3 różne typy zadań,
- używać głównie `amigo-codemap` i standardowych metod (`git`, `rg`, build/test),
- po każdym tasku dopisać krótki wynik i szacowane tokeny,
- na końcu porównać, które raporty dały największy zwrot.

Struktura:
- `001-symbol-migration`
- `002-file-ops-cleanup`
- `003-large-file-split`
- `summary.md`

Minimalny workflow dla każdego tasku:
1. uruchom komendy z `task.md`,
2. zanotuj użyte raporty i ręczne fallbacki,
3. zanotuj ile plików trzeba było realnie otworzyć,
4. zanotuj szacowane tokeny `used` i `saved`,
5. dopisz wynik do `result.md`.

Szacunek tokenów:
- `used`: ile poszło na realne prowadzenie tasku,
- `saved`: ile prawdopodobnie oszczędził codemap względem ręcznego `rg + git diff + czytanie plików`.

Rekomendowany format wpisu:

```text
- Used: ~2500
- Saved: ~6000
- Reports: impact, open-set, slice, verify-plan
- Manual fallback: git diff --stat, rg -n ...
- Files opened: 4
- Notes: ...
```
