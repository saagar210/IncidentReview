import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from '../locales/en.json';
import es from '../locales/es.json';

// Get user's preferred locale from localStorage or browser
const getInitialLanguage = (): string => {
  // Check localStorage first
  const stored = localStorage.getItem('app-locale');
  if (stored) return stored;

  // Check browser language
  const browserLang = navigator.language.split('-')[0];
  if (browserLang === 'es') return 'es';
  if (browserLang === 'en') return 'en';

  // Default to English
  return 'en';
};

const resources = {
  en: { translation: en },
  es: { translation: es },
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: getInitialLanguage(),
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React already protects against XSS
    },
  });

// Persist locale changes to localStorage
i18n.on('languageChanged', (lng) => {
  localStorage.setItem('app-locale', lng);
});

export default i18n;
export type { };
