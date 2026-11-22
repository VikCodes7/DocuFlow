// All UI text lives in this folder (AD-10). Components use `t("key")`, never
// hard-coded strings. Adding a language means adding a JSON file here.
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en.json";

export const defaultNS = "translation";
export const resources = { en: { translation: en } } as const;

void i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  interpolation: { escapeValue: false }, // React already escapes.
});

export default i18n;
