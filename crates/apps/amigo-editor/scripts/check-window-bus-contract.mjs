import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const rustNames = readFileSync(resolve(root, "src-tauri/src/events/names.rs"), "utf8");
const tsBus = readFileSync(resolve(root, "src/app/windowBus.ts"), "utf8");
const tsContract = readFileSync(resolve(root, "src/app/windowBusContract.ts"), "utf8");

const rustEvents = [...rustNames.matchAll(/pub const [A-Z_]+: &str = "([^"]+)";/g)].map((match) => match[1]);
const tsEvents = [...tsBus.matchAll(/:\s*"([^"]+)"/g)].map((match) => match[1]);
const tsContractEvents = [...tsContract.matchAll(/"([^"]+)"/g)].map((match) => match[1]);

const missingInTs = rustEvents.filter((event) => !tsEvents.includes(event));
const missingInContract = rustEvents.filter((event) => !tsContractEvents.includes(event));

if (missingInTs.length > 0) {
  console.error("Window bus events missing in src/app/windowBus.ts:");
  for (const event of missingInTs) {
    console.error(`- ${event}`);
  }
  process.exit(1);
}

if (missingInContract.length > 0) {
  console.error("Backend window bus events missing in src/app/windowBusContract.ts:");
  for (const event of missingInContract) {
    console.error(`- ${event}`);
  }
  process.exit(1);
}

console.log(`Window bus contract check passed (${rustEvents.length} backend events).`);
