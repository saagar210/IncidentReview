import { useTranslation } from 'react-i18next';

export function useLocale() {
  const { i18n, t } = useTranslation();

  return {
    currentLocale: i18n.language,
    availableLocales: ['en', 'es'] as const,
    changeLocale: (locale: string) => i18n.changeLanguage(locale),
    t,
  };
}
