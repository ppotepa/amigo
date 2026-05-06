const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

export function normalizeSlugId(value: string): string {
  return value
    .trim()
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[łŁ]/g, "l")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
}

export function validateSlugId(value: string, exists: boolean): {
  normalized: string;
  error: string | null;
  invalid: boolean;
} {
  const normalized = normalizeSlugId(value);
  if (!normalized) {
    return { normalized, invalid: true, error: "Id is required." };
  }
  if (!SLUG_PATTERN.test(normalized)) {
    return {
      normalized,
      invalid: true,
      error: "Id must use lowercase letters, numbers, and dashes.",
    };
  }
  if (exists) {
    return { normalized, invalid: true, error: "Id already exists." };
  }
  return { normalized, invalid: false, error: null };
}

export function nextAvailableSlugId(
  base: string,
  exists: (candidate: string) => boolean,
  startAt = 2,
): string {
  const normalizedBase = normalizeSlugId(base) || "new-item";
  if (!exists(normalizedBase)) return normalizedBase;
  let index = startAt;
  while (exists(`${normalizedBase}-${index}`)) {
    index += 1;
  }
  return `${normalizedBase}-${index}`;
}
