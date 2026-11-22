import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CoreError, HistoryEntry } from "../../bindings";
import { Page } from "../../components/Page";
import { clearHistory, listHistory } from "../../services/app";
import { errorMessage } from "../../services/errors";

export default function HistoryPage() {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<HistoryEntry[] | null>(null);
  const [error, setError] = useState<CoreError | null>(null);

  useEffect(() => {
    let active = true;
    void listHistory().then((res) => {
      if (!active) return;
      if (res.ok) setEntries(res.data);
      else setError(res.error);
    });
    return () => {
      active = false;
    };
  }, []);

  const onClear = async () => {
    const res = await clearHistory();
    if (res.ok) setEntries([]);
    else setError(res.error);
  };

  return (
    <Page title={t("history.title")}>
      {error && <p role="alert" className="text-red-600">{errorMessage(t, error)}</p>}
      {entries === null ? (
        <p>{t("common.loading")}</p>
      ) : entries.length === 0 ? (
        <p className="text-neutral-600 dark:text-neutral-400">{t("history.empty")}</p>
      ) : (
        <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
          {entries.map((entry) => (
            <li key={entry.id} className="py-2">
              {entry.operation} · {entry.sourceNames.join(", ")}
            </li>
          ))}
        </ul>
      )}
      <button
        type="button"
        onClick={() => void onClear()}
        className="mt-6 rounded-md border border-neutral-300 px-3 py-1.5 text-sm hover:bg-neutral-100 dark:border-neutral-600 dark:hover:bg-neutral-800"
      >
        {t("history.clear")}
      </button>
      <p className="mt-2 text-sm text-neutral-500">{t("history.clearNote")}</p>
    </Page>
  );
}
