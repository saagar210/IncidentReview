import { useLocale } from "../lib/useLocale";
import "./LocaleSwitcher.css";

export function LocaleSwitcher() {
  const { currentLocale, availableLocales, changeLocale } = useLocale();

  return (
    <div className="locale-switcher">
      <label htmlFor="locale-select">Language:</label>
      <select
        id="locale-select"
        value={currentLocale}
        onChange={(e) => changeLocale(e.target.value)}
        className="locale-select"
      >
        <option value="en">English</option>
        <option value="es">Español</option>
      </select>
    </div>
  );
}
