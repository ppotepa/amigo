const MAX_SAMPLE_SIZE = 180;
const ALPHA_THRESHOLD = 24;

const state = {
  file: null,
  colors: [],
  imageUrl: null,
  image: null,
  reducedUrl: null,
  reducedBlob: null,
};

const elements = {
  toolButtons: document.querySelectorAll(".tool-button"),
  paletteTool: document.querySelector("#palette-tool"),
  placeholderTool: document.querySelector("#placeholder-tool"),
  placeholderTitle: document.querySelector("#placeholder-title"),
  imageInput: document.querySelector("#image-input"),
  dropzone: document.querySelector("#dropzone"),
  previewImage: document.querySelector("#preview-image"),
  colorCount: document.querySelector("#color-count"),
  colorCountOutput: document.querySelector("#color-count-output"),
  precisionMode: document.querySelector("#precision-mode"),
  sortMode: document.querySelector("#sort-mode"),
  previewReduced: document.querySelector("#preview-reduced"),
  paletteList: document.querySelector("#palette-list"),
  status: document.querySelector("#status"),
  sourceName: document.querySelector("#source-name"),
  sampleSize: document.querySelector("#sample-size"),
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

elements.colorCount.addEventListener("input", () => {
  elements.colorCountOutput.textContent = elements.colorCount.value;
  if (state.file) loadImageFile(state.file, { keepPreviewMode: true });
});

elements.precisionMode.addEventListener("change", () => {
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
    const result = extractPalette(image, Number(elements.colorCount.value), Number(elements.precisionMode.value));
    state.colors = result.colors;
    elements.sampleSize.textContent = `${result.sampledPixels.toLocaleString()} px`;
    await rebuildReducedPreview();
    renderPalette();
    if (!options.keepPreviewMode) elements.previewReduced.checked = false;
    updatePreviewMode();
    setStatus(result.colors.length ? "Palette ready." : "No opaque pixels found.");
  } catch (error) {
    state.colors = [];
    state.image = null;
    clearReducedPreview();
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

function extractPalette(image, colorCount, channelStep) {
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

    const rgb = [
      quantizeChannel(imageData.data[index], channelStep),
      quantizeChannel(imageData.data[index + 1], channelStep),
      quantizeChannel(imageData.data[index + 2], channelStep),
    ];
    const key = rgb.join(",");
    const bucket = buckets.get(key);
    if (bucket) {
      bucket.count += 1;
    } else {
      buckets.set(key, { rgb, count: 1 });
    }
    sampledPixels += 1;
  }

  const colors = Array.from(buckets.values())
    .sort((left, right) => right.count - left.count)
    .slice(0, colorCount)
    .map((bucket) => ({
      hex: rgbToHex(bucket.rgb),
      rgb: bucket.rgb,
      count: bucket.count,
      share: sampledPixels > 0 ? bucket.count / sampledPixels : 0,
    }));

  return { colors, sampledPixels };
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

function rgbToHex([red, green, blue]) {
  return `#${hexPair(red)}${hexPair(green)}${hexPair(blue)}`;
}

function hexPair(value) {
  return Math.max(0, Math.min(255, value)).toString(16).padStart(2, "0").toUpperCase();
}

function luminance([red, green, blue]) {
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
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
