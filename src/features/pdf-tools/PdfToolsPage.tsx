import { useTranslation } from "react-i18next";
import { Page } from "../../components/Page";

export default function PdfToolsPage() {
  const { t } = useTranslation();
  return <Page title={t("pdfTools.title")} description={t("pdfTools.description")} />;
}
