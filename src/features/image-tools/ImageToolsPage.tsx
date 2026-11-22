import { useTranslation } from "react-i18next";
import { Page } from "../../components/Page";

export default function ImageToolsPage() {
  const { t } = useTranslation();
  return <Page title={t("imageTools.title")} description={t("imageTools.description")} />;
}
