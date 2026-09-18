import type { WorkSummary } from "./types";

export function formatDuration(secs: number): string {
  if (secs < 60) return `${secs} 秒`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  if (s === 0) return `${m} 分`;
  return `${m} 分 ${s} 秒`;
}

export function summaryTitle(s: WorkSummary): string {
  const total = s.workSecs + s.idleSecs;
  return `专注 ${formatDuration(total)}`;
}

export function summaryKeys(s: WorkSummary): string {
  const keys =
    s.keystrokes > 0 ? `敲了约 ${s.keystrokes.toLocaleString("zh-CN")} 次键` : "这一轮几乎没怎么打字";
  if (s.idleSecs >= 60) {
    return `${keys}，发呆 ${formatDuration(s.idleSecs)}`;
  }
  return keys;
}

export function fillSummary(
  root: HTMLElement,
  s: WorkSummary | null | undefined,
): void {
  if (!s || (s.workSecs === 0 && s.idleSecs === 0)) {
    root.hidden = true;
    return;
  }
  root.hidden = false;
  const title = root.querySelector("[data-summary-title]");
  const list = root.querySelector("[data-summary-apps]");
  const keys = root.querySelector("[data-summary-keys]");
  if (title) title.textContent = summaryTitle(s);
  if (keys) keys.textContent = summaryKeys(s);
  if (!list) return;

  const max = Math.max(1, ...s.apps.map((a) => a.secs));
  list.innerHTML = s.apps
    .map((app) => {
      const pct = Math.max(6, Math.round((app.secs / max) * 100));
      const sub = app.title && app.title !== app.name
        ? `<span class="sum-title">${escapeHtml(app.title)}</span>`
        : "";
      return `<li>
        <div class="sum-row">
          <span class="sum-name">${escapeHtml(app.name)}</span>
          <span class="sum-time">${formatDuration(app.secs)}</span>
        </div>
        ${sub}
        <i class="sum-bar" style="width:${pct}%"></i>
      </li>`;
    })
    .join("");
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
