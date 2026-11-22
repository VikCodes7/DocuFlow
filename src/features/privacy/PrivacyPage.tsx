import { useTranslation } from "react-i18next";
import { Page } from "../../components/Page";

const POINTS = ["onDevice", "noAccount", "noTelemetry", "history", "installer"] as const;

export default function PrivacyPage() {
  const { t } = useTranslation();
  return (
    <Page title={t("privacy.title")} description={t("app.tagline")}>
      <ul className="list-disc space-y-2 pl-5">
        {POINTS.map((key) => (
          <li key={key}>{t(`privacy.points.${key}`)}</li>
        ))}
      </ul>
    </Page>
  );
}
