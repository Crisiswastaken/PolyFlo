export function formatHistoryTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();

  const timeStr = date.toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });

  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const startOfYesterday = new Date(startOfToday);
  startOfYesterday.setDate(startOfYesterday.getDate() - 1);

  if (date >= startOfToday) {
    return `Today, ${timeStr}`;
  }
  if (date >= startOfYesterday) {
    return `Yesterday, ${timeStr}`;
  }

  const dayStr = date.toLocaleDateString(undefined, {
    weekday: "short",
  });
  return `${dayStr}, ${timeStr}`;
}
