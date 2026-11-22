// Thin wrappers over the generated bindings. Components import from here,
// so they don't depend on the binding layout directly.
import { commands, type AppInfo, type CoreError, type HistoryEntry } from "../bindings";

export type Result<T> = { ok: true; data: T } | { ok: false; error: CoreError };

function unwrap<T>(res: { status: "ok"; data: T } | { status: "error"; error: CoreError }): Result<T> {
  return res.status === "ok" ? { ok: true, data: res.data } : { ok: false, error: res.error };
}

export async function getAppInfo(): Promise<AppInfo> {
  return commands.appInfo();
}

export async function listHistory(): Promise<Result<HistoryEntry[]>> {
  return unwrap(await commands.listHistory());
}

export async function clearHistory(): Promise<Result<null>> {
  return unwrap(await commands.clearHistory());
}
