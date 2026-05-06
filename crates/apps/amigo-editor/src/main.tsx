import { installConsoleLogFile } from "./debug/consoleLogFile";

installConsoleLogFile();

void import("./mainApp").catch((error: unknown) => {
  console.error("Failed to bootstrap Amigo Editor", error);
});
