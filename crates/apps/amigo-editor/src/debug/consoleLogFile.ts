import { invoke } from "@tauri-apps/api/core";

type ConsoleLogLevel = "log" | "info" | "warn" | "error" | "debug" | "trace";

type ConsoleLogEntry = {
  timestamp: string;
  level: ConsoleLogLevel | "unhandledrejection";
  message: string;
  args: string[];
  source?: string | null;
};

const CONSOLE_LEVELS: ConsoleLogLevel[] = ["log", "info", "warn", "error", "debug", "trace"];
const MAX_ARG_LENGTH = 12_000;
const MAX_BATCH_SIZE = 80;
const FLUSH_DELAY_MS = 200;

let installed = false;
let disabled = false;
let flushTimer = 0;
let flushing = false;
const pendingEntries: ConsoleLogEntry[] = [];

export function installConsoleLogFile() {
  if (installed) return;
  installed = true;

  for (const level of CONSOLE_LEVELS) {
    const original = console[level].bind(console);
    console[level] = (...args: unknown[]) => {
      original(...args);
      enqueueConsoleLog(level, args, currentSource());
    };
  }

  window.addEventListener("error", (event) => {
    enqueueConsoleLog(
      "error",
      [
        event.message,
        event.error instanceof Error ? (event.error.stack ?? event.error.message) : event.error,
      ],
      `${event.filename}:${event.lineno}:${event.colno}`,
    );
  });

  window.addEventListener("unhandledrejection", (event) => {
    enqueueConsoleLog("unhandledrejection", ["Unhandled promise rejection", event.reason], currentSource());
  });

  enqueueConsoleLog("info", ["Console file logging installed"], currentSource());
}

function enqueueConsoleLog(level: ConsoleLogEntry["level"], args: unknown[], source?: string | null) {
  if (disabled) return;

  const serializedArgs = args.map((arg) => serializeConsoleArg(arg));
  pendingEntries.push({
    timestamp: new Date().toISOString(),
    level,
    message: serializedArgs[0] ?? "",
    args: serializedArgs,
    source,
  });

  if (pendingEntries.length >= MAX_BATCH_SIZE) {
    void flushConsoleLogs();
    return;
  }

  if (flushTimer) return;
  flushTimer = window.setTimeout(() => {
    flushTimer = 0;
    void flushConsoleLogs();
  }, FLUSH_DELAY_MS);
}

async function flushConsoleLogs() {
  if (disabled || flushing || pendingEntries.length === 0) return;
  flushing = true;
  const entries = pendingEntries.splice(0, MAX_BATCH_SIZE);

  try {
    await invoke("append_editor_console_logs", { entries });
  } catch {
    disabled = true;
    pendingEntries.length = 0;
  } finally {
    flushing = false;
    if (!disabled && pendingEntries.length > 0) {
      void flushConsoleLogs();
    }
  }
}

function serializeConsoleArg(value: unknown): string {
  if (typeof value === "string") return limitText(value);
  if (value instanceof Error) return limitText(value.stack ?? value.message);
  if (typeof value === "undefined") return "undefined";
  if (value === null) return "null";

  try {
    return limitText(JSON.stringify(value, jsonReplacer(), 2));
  } catch {
    return limitText(String(value));
  }
}

function jsonReplacer() {
  const seen = new WeakSet<object>();
  return (_key: string, value: unknown) => {
    if (typeof value === "bigint") return `${value}n`;
    if (value instanceof Error) return value.stack ?? value.message;
    if (typeof value !== "object" || value === null) return value;
    if (seen.has(value)) return "[Circular]";
    seen.add(value);
    return value;
  };
}

function limitText(value: string): string {
  if (value.length <= MAX_ARG_LENGTH) return value;
  return `${value.slice(0, MAX_ARG_LENGTH)}... [truncated ${value.length - MAX_ARG_LENGTH} chars]`;
}

function currentSource(): string {
  return `${window.location.pathname}${window.location.hash}`;
}
