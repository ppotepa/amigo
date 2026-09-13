import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { CompanionBootstrap } from './contracts';

export interface CompanionHost {
  canPickFile: boolean;
  nativeError: string | null;
  invoke<T>(command: string, args: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
}
export function companionHost(boot: CompanionBootstrap): CompanionHost {
  const tauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  return {
    canPickFile: tauri,
    nativeError: tauri ? boot.native_error ?? null : 'Przeglądarka: dostępna prezentacja JPEG.',
    invoke: (command, args) => invoke(command, args),
    listen<T>(event: string, callback: (payload: T) => void) { return listen<T>(event, message => callback(message.payload)); },
  };
}
