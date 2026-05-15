# Third-Party Notices

This repository contains original code and content, plus implementations that were
developed with direct reference to upstream open-source work.

## RainGlass effect reference

The `RainGlass` post-fx implementation in:

- `crates/engine/render-wgpu/src/renderer/service/post_fx/rain_glass/`
- `crates/2d/post-fx/src/`
- `mods/rotten-club/scripts/packages/fx/rain_glass_presets.rhai`

was tuned and partially ported with reference to the upstream project:

- `SardineFish/raindrop-fx`
- Repository: <https://github.com/SardineFish/raindrop-fx>
- License: MIT

The upstream project README also states that it is inspired by:

- `codrops/RainEffect`
- Repository: <https://github.com/codrops/RainEffect>

Amigo does not bundle the upstream HTML demo files or npm package as runtime
dependencies. This notice exists to preserve attribution and license context for
the reference implementation and any adapted logic or shader behavior derived
from that work.

### Bundled upstream license text: `SardineFish/raindrop-fx`

```text
MIT License

Copyright (c) 2021 SardineFish

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
