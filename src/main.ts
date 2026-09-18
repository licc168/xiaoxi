import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FIGURE_HTML } from "./figure";
import { formatClock } from "./exercises";
import { playBell, speak, unlockAudio } from "./audio";
import { fillSummary } from "./summary";
import type { Settings, TimerSnapshot } from "./types";

const CIRC = 2 * Math.PI * 54;

const mascot = document.querySelector("#mascot") as HTMLElement;
mascot.innerHTML = FIGURE_HTML;

const timeEl = document.querySelector("#time") as HTMLElement;
const phaseEl = document.querySelector("#phase") as HTMLElement;
const progEl = document.querySelector("#prog") as SVGCircleElement;
const ringEl = document.querySelector("#ring") as HTMLElement;
const toggleBtn = document.querySelector("#toggle") as HTMLButtonElement;
const pauseBtn = document.querySelector("#pause") as HTMLButtonElement;
const workInput = document.querySelector("#workMinutes") as HTMLInputElement;
const restInput = document.querySelector("#restMinutes") as HTMLInputElement;
const voiceInput = document.querySelector("#voice") as HTMLInputElement;
const beatInput = document.querySelector("#beat") as HTMLInputElement;
const strictInput = document.querySelector("#strict") as HTMLInputElement;
const summaryInput = document.querySelector("#summary") as HTMLInputElement;
const lastSummaryEl = document.querySelector("#lastSummary") as HTMLElement;

let snap: TimerSnapshot | null = null;
let saving = false;

function settingsFromForm(): Settings {
  return {
    workMinutes: clamp(Number(workInput.value) || 45, 1, 180),
    restMinutes: clamp(Number(restInput.value) || 10, 1, 60),
    voice: voiceInput.checked,
    beat: beatInput.checked,
    strict: strictInput.checked,
    summary: summaryInput.checked,
  };
}

function clamp(n: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, n));
}

function applyForm(s: Settings) {
  workInput.value = String(s.workMinutes);
  restInput.value = String(s.restMinutes);
  voiceInput.checked = s.voice;
  beatInput.checked = s.beat;
  strictInput.checked = s.strict;
  summaryInput.checked = s.summary;
  document.querySelectorAll("#presets button").forEach((btn) => {
    const b = btn as HTMLButtonElement;
    b.classList.toggle(
      "active",
      Number(b.dataset.work) === s.workMinutes && Number(b.dataset.rest) === s.restMinutes,
    );
  });
}

function render(next: TimerSnapshot) {
  snap = next;
  applyForm(next.settings);
  timeEl.textContent = formatClock(next.remainingSecs);
  ringEl.classList.toggle("rest", next.phase === "rest");
  const total = Math.max(1, next.totalSecs);
  const progress = 1 - next.remainingSecs / total;
  progEl.style.strokeDasharray = String(CIRC);
  progEl.style.strokeDashoffset = String(CIRC * (1 - progress));

  if (next.phase === "idle") {
    phaseEl.textContent = "准备开始";
    toggleBtn.textContent = "开始专注";
    pauseBtn.disabled = true;
  } else if (next.paused) {
    phaseEl.textContent = next.phase === "rest" ? "休息已暂停" : "专注已暂停";
    toggleBtn.textContent = "继续";
    pauseBtn.disabled = true;
  } else if (next.phase === "work") {
    phaseEl.textContent = "专注中";
    toggleBtn.textContent = "暂停";
    pauseBtn.disabled = false;
  } else {
    phaseEl.textContent = "课间操进行中";
    toggleBtn.textContent = "暂停";
    pauseBtn.disabled = false;
  }
  fillSummary(lastSummaryEl, next.lastSummary);
}

async function persist() {
  if (saving) return;
  saving = true;
  try {
    const next = await invoke<TimerSnapshot>("save_settings", {
      settings: settingsFromForm(),
    });
    render(next);
  } finally {
    saving = false;
  }
}

async function refresh() {
  const next = await invoke<TimerSnapshot>("get_state");
  render(next);
}

document.querySelectorAll(".stepper button").forEach((btn) => {
  btn.addEventListener("click", () => {
    const field = (btn as HTMLButtonElement).dataset.field as "workMinutes" | "restMinutes";
    const delta = Number((btn as HTMLButtonElement).dataset.delta);
    const input = field === "workMinutes" ? workInput : restInput;
    const max = field === "workMinutes" ? 180 : 60;
    input.value = String(clamp((Number(input.value) || 0) + delta, 1, max));
    void persist();
  });
});

[workInput, restInput].forEach((input) => {
  input.addEventListener("change", () => void persist());
});
[voiceInput, beatInput, strictInput, summaryInput].forEach((input) => {
  input.addEventListener("change", () => void persist());
});

document.querySelector("#presets")?.addEventListener("click", (e) => {
  const btn = (e.target as HTMLElement).closest("button") as HTMLButtonElement | null;
  if (!btn) return;
  workInput.value = btn.dataset.work ?? "45";
  restInput.value = btn.dataset.rest ?? "10";
  void persist();
});

toggleBtn.addEventListener("click", async () => {
  await unlockAudio();
  if (!snap || snap.phase === "idle") {
    render(await invoke<TimerSnapshot>("start"));
    return;
  }
  if (snap.paused) {
    render(await invoke<TimerSnapshot>("start"));
    return;
  }
  render(await invoke<TimerSnapshot>("pause"));
});

pauseBtn.addEventListener("click", async () => {
  render(await invoke<TimerSnapshot>("pause"));
});

document.querySelector("#reset")?.addEventListener("click", async () => {
  render(await invoke<TimerSnapshot>("reset"));
});

document.querySelector("#restNow")?.addEventListener("click", async () => {
  await unlockAudio();
  render(await invoke<TimerSnapshot>("rest_now", { secs: null }));
});

document.querySelector("#preview")?.addEventListener("click", async () => {
  await unlockAudio();
  render(await invoke<TimerSnapshot>("rest_now", { secs: 45 }));
});

window.addEventListener("keydown", (e) => {
  if (e.code === "Space" && e.target === document.body) {
    e.preventDefault();
    toggleBtn.click();
  }
});

await refresh();

await listen<TimerSnapshot>("timer-tick", (event) => {
  render(event.payload);
});

await listen<TimerSnapshot>("rest-begin", (event) => {
  render(event.payload);
  playBell("start");
  if (event.payload.settings.voice) {
    speak("休息时间到了，跟我做课间操。");
  }
});

await listen<TimerSnapshot>("rest-end", (event) => {
  render(event.payload);
  playBell("end");
  if (event.payload.settings.voice && event.payload.phase === "work") {
    speak("休息结束，继续工作吧。");
  }
});
