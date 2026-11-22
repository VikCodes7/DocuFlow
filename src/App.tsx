import { lazy, Suspense, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type View, useAppStore } from "./lib/store";
import { getAppInfo } from "./services/app";

// Feature screens load on demand for a faster startup (doc §9.2).
const screens = {
  imageTools: lazy(() => import("./features/image-tools/ImageToolsPage")),
  pdfTools: lazy(() => import("./features/pdf-tools/PdfToolsPage")),
  history: lazy(() => import("./features/history/HistoryPage")),
  privacy: lazy(() => import("./features/privacy/PrivacyPage")),
} satisfies Record<View, unknown>;

const NAV: View[] = ["imageTools", "pdfTools", "history", "privacy"];

export function App() {
  const { t } = useTranslation();
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppInfo()
      .then((info) => setVersion(info.version))
      .catch(() => setVersion(null));
  }, []);

  const Screen = screens[view];

  return (
    <div className="flex h-full">
      <nav className="flex w-60 shrink-0 flex-col border-r border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950">
        <div className="mb-6 px-2 text-lg font-semibold">{t("app.name")}</div>
        <ul className="space-y-1">
          {NAV.map((item) => (
            <li key={item}>
              <button
                type="button"
                aria-current={view === item ? "page" : undefined}
                onClick={() => setView(item)}
                className="w-full rounded-md px-2 py-1.5 text-left text-sm hover:bg-neutral-100 aria-[current=page]:bg-neutral-200 dark:hover:bg-neutral-800 dark:aria-[current=page]:bg-neutral-800"
              >
                {t(`nav.${item}`)}
              </button>
            </li>
          ))}
        </ul>
        {version && <div className="mt-auto px-2 text-xs text-neutral-500">{t("app.version", { version })}</div>}
      </nav>
      <main className="flex-1 overflow-y-auto">
        <Suspense fallback={<p className="p-8">{t("common.loading")}</p>}>
          <Screen />
        </Suspense>
      </main>
    </div>
  );
}
