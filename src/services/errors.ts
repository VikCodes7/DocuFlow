import type { TFunction } from "i18next";
import type { CoreError } from "../bindings";

/** Maps a backend error code to a translated, user-facing message (AD-3). */
export function errorMessage(t: TFunction, error: CoreError): string {
  return t(`errors.${error.code}`);
}
