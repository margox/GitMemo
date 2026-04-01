/**
 * Parse YAML/frontmatter dates: ISO with offset (`...T...+08:00`, `Z`) or legacy `YYYY-MM-DD HH:mm:ss`.
 */
export function parseGitMemoDate(dateStr: string): Date | null {
  const s = dateStr.trim();
  if (!s) return null;
  let candidate = s;
  // `YYYY-MM-DD HH:mm...` without `T` → normalize for Date (ISO with offset usually already has `T`)
  if (!s.includes("T") && /^\d{4}-\d{2}-\d{2} \d/.test(s)) {
    candidate = s.replace(" ", "T");
  }
  const d = new Date(candidate);
  return Number.isNaN(d.getTime()) ? null : d;
}

/**
 * Format a date string as relative time.
 * Pass a translation function `t` for localized output.
 */
export function relativeTime(
  dateStr: string,
  t?: (key: string, ...args: (string | number)[]) => string
): string {
  try {
    const d = parseGitMemoDate(dateStr);
    if (!d) return dateStr.slice(0, 16);
    const diff = Math.floor((Date.now() - d.getTime()) / 1000);
    if (diff < 0) return dateStr.slice(0, 16);
    const tr = t || ((key: string, ...args: (string | number)[]) => {
      // Fallback English
      const map: Record<string, string> = {
        "time.justNow": "Just now",
        "time.minAgo": "{0} min ago",
        "time.hrAgo": "{0} hr ago",
        "time.dayAgo": "{0} day ago",
      };
      let s = map[key] || key;
      args.forEach((a, i) => { s = s.replace(`{${i}}`, String(a)); });
      return s;
    });
    if (diff < 60) return tr("time.justNow");
    if (diff < 3600) return tr("time.minAgo", Math.floor(diff / 60));
    if (diff < 86400) return tr("time.hrAgo", Math.floor(diff / 3600));
    if (diff < 604800) return tr("time.dayAgo", Math.floor(diff / 86400));
    return dateStr.slice(0, 16);
  } catch {
    return dateStr;
  }
}
