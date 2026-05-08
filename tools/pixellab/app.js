const MAX_SAMPLE_SIZE = 180;
const ALPHA_THRESHOLD = 24;

const state = {
  file: null,
  colors: [],
  imageUrl: null,
  image: null,
  reducedUrl: null,
  reducedBlob: null,
  colorCountMode: "fixed",
};

const elements = {
  toolButtons: document.querySelectorAll(".tool-button"),
  paletteTool: document.querySelector("#palette-tool"),
  placeholderTool: document.querySelector("#placeholder-tool"),
  placeholderTitle: document.querySelector("#placeholder-title"),
  imageInput: document.querySelector("#image-input"),
  dropzone: document.querySelector("#dropzone"),
  previewImage: document.querySelector("#preview-image"),
  presetButtons: document.querySelectorAll(".preset-button"),
  customColorCount: document.querySelector("#custom-color-count"),
  strategyMode: document.querySelector("#strategy-mode"),
  precisionMode: document.querySelector("#precision-mode"),
  sortMode: document.querySelector("#sort-mode"),
  previewReduced: document.querySelector("#preview-reduced"),
  mergeSimilar: document.querySelector("#merge-similar"),
  ignoreNeutrals: document.querySelector("#ignore-neutrals"),
  paletteList: document.querySelector("#palette-list"),
  status: document.querySelector("#status"),
  sourceName: document.querySelector("#source-name"),
  sampleSize: document.querySelector("#sample-size"),
  detectedCount: document.querySelector("#detected-count"),
  copyHex: document.querySelector("#copy-hex"),
  copyCss: document.querySelector("#copy-css"),
  copyJson: document.querySelector("#copy-json"),
  downloadPalette: document.querySelector("#download-palette"),
  downloadPng: document.querySelector("#download-png"),
  resetPreview: document.querySelector("#reset-preview"),
};

elements.toolButtons.forEach((button) => {
  button.addEventListener("click", () => selectTool(button));
});

elements.imageInput.addEventListener("change", () => {
  const file = elements.imageInput.files[0];
  if (file) loadImageFile(file);
});

elements.presetButtons.forEach((button) => {
  button.addEventListener("click", () => {
    elements.presetButtons.forEach((candidate) => candidate.classList.toggle("selected", candidate === button));
    state.colorCountMode = button.dataset.count === "auto" ? "auto" : "fixed";
    if (button.dataset.count !== "auto") elements.customColorCount.value = button.dataset.count;
    if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
  });
});

elements.customColorCount.addEventListener("change", () => {
  state.colorCountMode = "fixed";
  elements.presetButtons.forEach((button) => button.classList.toggle("selected", button.dataset.count === elements.customColorCount.value));
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});

elements.strategyMode.addEventListener("change", () => {
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});
elements.precisionMode.addEventListener("change", () => {
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});
elements.mergeSimilar.addEventListener("change", () => {
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});
elements.ignoreNeutrals.addEventListener("change", () => {
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});
elements.sortMode.addEventListener("change", renderPalette);
elements.previewReduced.addEventListener("change", updatePreviewMode);
elements.copyHex.addEventListener("click", () => copyPalette("hex"));
elements.copyCss.addEventListener("click", () => copyPalette("css"));
elements.copyJson.addEventListener("click", () => copyPalette("json"));
elements.downloadPalette.addEventListener("click", downloadPalette);
elements.downloadPng.addEventListener("click", downloadReducedPng);
elements.resetPreview.addEventListener("click", () => {
  elements.previewReduced.checked = false;
  updatePreviewMode();
});

document.addEventListener("paste", (event) => {
  const file =
    Array.from(event.clipboardData?.files ?? []).find((candidate) => candidate.type.startsWith("image/")) ??
    Array.from(event.clipboardData?.items ?? [])
      .find((candidate) => candidate.type.startsWith("image/"))
      ?.getAsFile();
  if (file) {
    event.preventDefault();
    loadImageFile(file);
  }
});

["dragenter", "dragover"].forEach((eventName) => {
  elements.dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    elements.dropzone.classList.add("dragging");
  });
});

["dragleave", "drop"].forEach((eventName) => {
  elements.dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    elements.dropzone.classList.remove("dragging");
  });
});

elements.dropzone.addEventListener("drop", (event) => {
  const file = Array.from(event.dataTransfer.files).find((candidate) => candidate.type.startsWith("image/"));
  if (file) loadImageFile(file);
});

function selectTool(button) {
  elements.toolButtons.forEach((candidate) => candidate.classList.toggle("selected", candidate === button));

  const tool = button.dataset.tool;
  if (tool === "palette") {
    elements.paletteTool.classList.add("active");
    elements.placeholderTool.classList.remove("active");
    return;
  }

  elements.paletteTool.classList.remove("active");
  elements.placeholderTool.classList.add("active");
  elements.placeholderTitle.textContent = button.querySelector("strong").textContent;
}

async function loadImageFile(file, options = {}) {
  state.file = file;
  setStatus("Extracting palette...");
  elements.sourceName.textContent = file.name;

  if (state.imageUrl) URL.revokeObjectURL(state.imageUrl);
  const objectUrl = URL.createObjectURL(file);
  state.imageUrl = objectUrl;
  elements.previewImage.src = objectUrl;
  elements.previewImage.hidden = false;
  elements.dropzone.classList.add("has-image");
  elements.dropzone.querySelector(".dropzone-empty").hidden = true;

  try {
    const image = await loadImage(objectUrl);
    state.image = image;
    const result = extractPalette(image, extractionOptions());
    state.colors = result.colors;
    elements.sampleSize.textContent = `${result.sampledPixels.toLocaleString()} px`;
    elements.detectedCount.textContent = `${result.detectedCount.toLocaleString()} colors`;
    await rebuildReducedPreview();
    renderPalette();
    if (!options.keepPreviewMode) elements.previewReduced.checked = false;
    updatePreviewMode();
    setStatus(result.colors.length ? paletteReadyMessage(result.colors.length) : "No opaque pixels found.");
  } catch (error) {
    state.colors = [];
    state.image = null;
    clearReducedPreview();
    elements.detectedCount.textContent = "0 colors";
    renderPalette();
    setStatus(error instanceof Error ? error.message : "Image extraction failed.");
  }
}

function loadImage(url) {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("Image could not be loaded."));
    image.src = url;
  });
}

function extractPalette(image, options) {
  const canvas = document.createElement("canvas");
  const scale = Math.min(1, MAX_SAMPLE_SIZE / Math.max(image.naturalWidth, image.naturalHeight));
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));

  const context = canvas.getContext("2d", { willReadFrequently: true });
  context.imageSmoothingEnabled = false;
  context.drawImage(image, 0, 0, canvas.width, canvas.height);

  const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
  const buckets = new Map();
  let sampledPixels = 0;

  for (let index = 0; index < imageData.data.length; index += 4) {
    const alpha = imageData.data[index + 3];
    if (alpha < ALPHA_THRESHOLD) continue;

    const sourceRgb = [
      imageData.data[index],
      imageData.data[index + 1],
      imageData.data[index + 2],
    ];
    if (options.ignoreNeutrals && isNearBlackOrWhite(sourceRgb)) continue;

    const rgb = options.strategy === "exact"
      ? sourceRgb
      : [
          quantizeChannel(sourceRgb[0], options.channelStep),
          quantizeChannel(sourceRgb[1], options.channelStep),
          quantizeChannel(sourceRgb[2], options.channelStep),
        ];
    const key = rgb.join(",");
    const bucket = buckets.get(key);
    if (bucket) {
      bucket.count += 1;
      bucket.sum[0] += sourceRgb[0];
      bucket.sum[1] += sourceRgb[1];
      bucket.sum[2] += sourceRgb[2];
    } else {
      buckets.set(key, { rgb, count: 1, sum: [...sourceRgb] });
    }
    sampledPixels += 1;
  }

  const candidates = Array.from(buckets.values()).map((bucket) => ({
    rgb: bucket.rgb,
    averageRgb: bucket.sum.map((value) => Math.round(value / bucket.count)),
    count: bucket.count,
    share: sampledPixels > 0 ? bucket.count / sampledPixels : 0,
  }));
  const colorCount = options.autoCount ? autoColorCount(candidates) : options.colorCount;
  const selected = selectPalette(candidates, colorCount, options);
  const colors = selected.map((bucket) => ({
    hex: rgbToHex(bucket.rgb),
    rgb: bucket.rgb,
    count: bucket.count,
    share: bucket.share,
  }));

  return { colors, sampledPixels, detectedCount: candidates.length };
}

function extractionOptions() {
  return {
    autoCount: state.colorCountMode === "auto",
    colorCount: clampNumber(Number(elements.customColorCount.value), 2, 4096),
    strategy: elements.strategyMode.value,
    channelStep: Number(elements.precisionMode.value),
    mergeSimilar: elements.mergeSimilar.checked,
    ignoreNeutrals: elements.ignoreNeutrals.checked,
  };
}

function selectPalette(candidates, colorCount, options) {
  if (candidates.length === 0) return [];

  const source = options.mergeSimilar && options.strategy !== "exact"
    ? mergeNearbyCandidates(candidates, options.strategy === "flat" ? 72 : 34)
    : candidates;
  const count = Math.min(colorCount, source.length);

  if (options.strategy === "average") {
    return source
      .map((candidate) => ({ ...candidate, rgb: candidate.averageRgb }))
      .sort((left, right) => right.count - left.count)
      .slice(0, count);
  }

  if (options.strategy === "balanced") {
    return pickDiverse(source, count, 48, (candidate) => candidate.count);
  }

  if (options.strategy === "contrast") {
    return pickDiverse(source, count, 72, (candidate) => luminance(candidate.rgb));
  }

  if (options.strategy === "accents") {
    return pickRareAccents(source, count);
  }

  if (options.strategy === "flat") {
    return pickDiverse(source, count, 84, (candidate) => candidate.count);
  }

  return source.sort((left, right) => right.count - left.count).slice(0, count);
}

function autoColorCount(candidates) {
  const sorted = [...candidates].sort((left, right) => right.count - left.count);
  let cumulative = 0;
  let count = 0;

  for (const candidate of sorted) {
    if (count >= 4 && candidate.share < 0.003) break;
    cumulative += candidate.share;
    count += 1;
    if (count >= 8 && cumulative >= 0.94) break;
    if (count >= 256) break;
  }

  return clampNumber(count || sorted.length, 2, 256);
}

function pickDiverse(candidates, count, minDistance, score) {
  const sorted = [...candidates].sort((left, right) => score(right) - score(left));
  const selected = [];

  for (const candidate of sorted) {
    if (selected.length >= count) break;
    if (selected.every((color) => colorDistance(color.rgb, candidate.rgb) >= minDistance)) {
      selected.push(candidate);
    }
  }

  for (const candidate of sorted) {
    if (selected.length >= count) break;
    if (!selected.includes(candidate)) selected.push(candidate);
  }

  return selected;
}

function pickRareAccents(candidates, count) {
  const dominantCount = Math.max(1, Math.ceil(count * 0.65));
  const dominant = [...candidates].sort((left, right) => right.count - left.count).slice(0, dominantCount);
  const accents = [...candidates]
    .filter((candidate) => !dominant.includes(candidate))
    .sort((left, right) => saturation(right.rgb) - saturation(left.rgb))
    .slice(0, count - dominant.length);

  return [...dominant, ...accents];
}

function mergeNearbyCandidates(candidates, threshold) {
  const groups = [];
  const sorted = [...candidates].sort((left, right) => right.count - left.count);

  for (const candidate of sorted) {
    const group = groups.find((entry) => colorDistance(entry.rgb, candidate.rgb) < threshold);
    if (!group) {
      groups.push({ ...candidate });
      continue;
    }
    const nextCount = group.count + candidate.count;
    group.rgb = weightedRgb(group.rgb, group.count, candidate.rgb, candidate.count);
    group.averageRgb = weightedRgb(group.averageRgb, group.count, candidate.averageRgb, candidate.count);
    group.count = nextCount;
    group.share += candidate.share;
  }

  return groups;
}

function weightedRgb(left, leftCount, right, rightCount) {
  const total = leftCount + rightCount;
  return [
    Math.round((left[0] * leftCount + right[0] * rightCount) / total),
    Math.round((left[1] * leftCount + right[1] * rightCount) / total),
    Math.round((left[2] * leftCount + right[2] * rightCount) / total),
  ];
}

function paletteReadyMessage(colorCount) {
  if (state.colorCountMode === "auto") return `Auto selected ${colorCount} colors.`;
  return "Palette ready.";
}

function renderPalette() {
  const colors = [...state.colors].sort((left, right) => {
    if (elements.sortMode.value === "luminance") return luminance(left.rgb) - luminance(right.rgb);
    if (elements.sortMode.value === "dominance") return right.count - left.count;
    return hueSortKey(left.rgb) - hueSortKey(right.rgb);
  });

  elements.paletteList.replaceChildren();
  elements.copyHex.disabled = colors.length === 0;
  elements.copyCss.disabled = colors.length === 0;
  elements.copyJson.disabled = colors.length === 0;
  elements.downloadPalette.disabled = colors.length === 0;
  elements.downloadPng.disabled = !state.reducedBlob;
  elements.resetPreview.disabled = !state.file;

  if (!colors.length) {
    const note = document.createElement("p");
    note.className = "empty-note";
    note.textContent = "No colors yet.";
    elements.paletteList.append(note);
    return;
  }

  colors.forEach((color) => {
    const button = document.createElement("button");
    button.className = "palette-color";
    button.type = "button";
    button.title = `Copy ${color.hex}`;
    button.addEventListener("click", () => {
      navigator.clipboard.writeText(color.hex);
      setStatus(`Copied ${color.hex}.`);
    });

    const swatch = document.createElement("span");
    swatch.className = "swatch";
    swatch.style.backgroundColor = color.hex;

    const code = document.createElement("code");
    code.textContent = color.hex;

    const share = document.createElement("small");
    share.textContent = `${(color.share * 100).toFixed(1)}%`;

    button.append(swatch, code, share);
    elements.paletteList.append(button);
  });
}

function copyPalette(format) {
  const text = paletteText(format);
  if (!text) return;
  navigator.clipboard.writeText(text);
  setStatus(`Copied ${format.toUpperCase()} palette.`);
}

function downloadPalette() {
  const text = paletteText("hex");
  if (!text) return;

  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = `${state.file?.name.replace(/\.[^.]+$/, "") || "pixellab-palette"}.txt`;
  link.click();
  URL.revokeObjectURL(link.href);
  setStatus("Exported HEX palette.");
}

function downloadReducedPng() {
  if (!state.reducedBlob) return;

  const link = document.createElement("a");
  link.href = URL.createObjectURL(state.reducedBlob);
  link.download = `${sourceBaseName()}-reduced.png`;
  link.click();
  URL.revokeObjectURL(link.href);
  setStatus("Exported reduced PNG.");
}

function paletteText(format) {
  const colors = sortedColors();
  if (format === "css") {
    return [":root {", ...colors.map((color, index) => `  --pixel-color-${index + 1}: ${color.hex};`), "}"].join("\n");
  }
  if (format === "json") {
    return JSON.stringify(colors.map((color) => ({
      hex: color.hex,
      rgb: color.rgb,
      share: Number(color.share.toFixed(4)),
    })), null, 2);
  }
  return colors.map((color) => color.hex).join("\n");
}

function sortedColors() {
  return [...state.colors].sort((left, right) => {
    if (elements.sortMode.value === "luminance") return luminance(left.rgb) - luminance(right.rgb);
    if (elements.sortMode.value === "dominance") return right.count - left.count;
    return hueSortKey(left.rgb) - hueSortKey(right.rgb);
  });
}

async function rebuildReducedPreview() {
  clearReducedPreview();
  if (!state.image || state.colors.length === 0) return;

  const canvas = document.createElement("canvas");
  canvas.width = state.image.naturalWidth;
  canvas.height = state.image.naturalHeight;

  const context = canvas.getContext("2d", { willReadFrequently: true });
  context.imageSmoothingEnabled = false;
  context.drawImage(state.image, 0, 0);

  const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
  for (let index = 0; index < imageData.data.length; index += 4) {
    if (imageData.data[index + 3] < ALPHA_THRESHOLD) continue;
    const nearest = nearestPaletteColor([
      imageData.data[index],
      imageData.data[index + 1],
      imageData.data[index + 2],
    ]);
    imageData.data[index] = nearest[0];
    imageData.data[index + 1] = nearest[1];
    imageData.data[index + 2] = nearest[2];
  }
  context.putImageData(imageData, 0, 0);

  state.reducedBlob = await new Promise((resolve) => canvas.toBlob(resolve, "image/png"));
  if (state.reducedBlob) state.reducedUrl = URL.createObjectURL(state.reducedBlob);
}

function clearReducedPreview() {
  if (state.reducedUrl) URL.revokeObjectURL(state.reducedUrl);
  state.reducedUrl = null;
  state.reducedBlob = null;
}

function updatePreviewMode() {
  if (elements.previewReduced.checked && state.reducedUrl) {
    elements.previewImage.src = state.reducedUrl;
    setStatus("Showing reduced preview.");
    return;
  }
  if (state.imageUrl) elements.previewImage.src = state.imageUrl;
}

function nearestPaletteColor(rgb) {
  let best = state.colors[0].rgb;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (const color of state.colors) {
    const distance =
      (rgb[0] - color.rgb[0]) ** 2 +
      (rgb[1] - color.rgb[1]) ** 2 +
      (rgb[2] - color.rgb[2]) ** 2;
    if (distance < bestDistance) {
      best = color.rgb;
      bestDistance = distance;
    }
  }
  return best;
}

function sourceBaseName() {
  return state.file?.name.replace(/\.[^.]+$/, "") || "pixellab";
}

function setStatus(message) {
  elements.status.textContent = message;
}

function quantizeChannel(value, channelStep) {
  return Math.min(255, Math.round(value / channelStep) * channelStep);
}

function clampNumber(value, min, max) {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.round(value)));
}

function rgbToHex([red, green, blue]) {
  return `#${hexPair(red)}${hexPair(green)}${hexPair(blue)}`;
}

function hexPair(value) {
  return Math.max(0, Math.min(255, value)).toString(16).padStart(2, "0").toUpperCase();
}

function luminance([red, green, blue]) {
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

function saturation([red, green, blue]) {
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  return max === 0 ? 0 : (max - min) / max;
}

function colorDistance(left, right) {
  return Math.sqrt(
    (left[0] - right[0]) ** 2 +
    (left[1] - right[1]) ** 2 +
    (left[2] - right[2]) ** 2,
  );
}

function isNearBlackOrWhite(rgb) {
  const light = luminance(rgb);
  return light < 10 || light > 245;
}

function hueSortKey([red, green, blue]) {
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const chroma = max - min;
  if (chroma === 0) return luminance([red, green, blue]);

  let hue;
  if (max === red) hue = 60 * (((green - blue) / chroma) % 6);
  else if (max === green) hue = 60 * ((blue - red) / chroma + 2);
  else hue = 60 * ((red - green) / chroma + 4);

  return (hue < 0 ? hue + 360 : hue) * 1000 + luminance([red, green, blue]);
}
