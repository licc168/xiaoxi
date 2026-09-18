import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FIGURE_HTML } from "./figure";
import {
  BEAT_MS,
  BEATS_PER_SET,
  ROUTINE,
  exerciseDurationMs,
  formatClock,
  type Exercise,
} from "./exercises";
import { playBeat, speak, stopSpeak, unlockAudio } from "./audio";
import { fillSummary } from "./summary";
import type { TimerSnapshot } from "./types";

const figureHost = document.querySelector("#figure") as HTMLElement;
figureHost.innerHTML = FIGURE_HTML;
const puppet = figureHost.querySelector(".puppet") as HTMLElement;
puppet.style.setProperty("--phrase", `${BEATS_PER_SET * BEAT_MS}ms`);

const remainEl = document.querySelector("#remain") as HTMLElement;
const sectionEl = document.querySelector("#section") as HTMLElement;
const nameEl = document.querySelector("#name") as HTMLElement;
const instructionEl = document.querySelector("#instruction") as HTMLElement;
const setLabelEl = document.querySelector("#setLabel") as HTMLElement;
const beatsEl = document.querySelector("#beats") as HTMLElement;
const actionsEl = document.querySelector("#actions") as HTMLElement;
const strictNote = document.querySelector("#strictNote") as HTMLElement;
const summaryEl = document.querySelector("#summary") as HTMLElement;

beatsEl.innerHTML = Array.from({ length: BEATS_PER_SET }, (_, i) => `<i>${i + 1}</i>`).join("");
const beatDots = [...beatsEl.querySelectorAll("i")];

let runId = 0;
let skipMove = false;
let stopAll = false;
let beatTimer = 0;
let restDeadline = 0;
let settingsVoice = true;
let settingsBeat = true;
let settingsStrict = false;

function setMove(move: string) {
  puppet.setAttribute("data-move", move);
}

function showExercise(ex: Exercise, setIndex: number, beatInSet: number) {
  sectionEl.textContent = ex.section;
  nameEl.textContent = ex.name;
  instructionEl.textContent = ex.instruction;
  setLabelEl.textContent = `第 ${setIndex + 1} / ${ex.sets} 组`;
  beatDots.forEach((dot, i) => {
    dot.classList.toggle("on", i === beatInSet);
    dot.classList.toggle("accent", i === 0 && beatInSet === 0);
  });
}

function planRoutine(totalMs: number): Exercise[] {
  const closing = ROUTINE.filter((e) => e.closing);
  const main = ROUTINE.filter((e) => !e.closing);
  const out: Exercise[] = [];
  let used = 0;
  const reserve = closing.reduce((sum, e) => sum + exerciseDurationMs(e), 0) + 4000;

  const pushFit = (ex: Exercise) => {
    const dur = exerciseDurationMs(ex);
    if (used + dur + reserve <= totalMs || out.length === 0) {
      out.push(ex);
      used += dur;
      return true;
    }
    return false;
  };

  for (const ex of main) {
    if (!pushFit(ex)) break;
  }
  let i = 1;
  while (used + reserve + exerciseDurationMs(main[i % main.length]) <= totalMs) {
    out.push(main[i % main.length]);
    used += exerciseDurationMs(main[i % main.length]);
    i += 1;
    if (i > 40) break;
  }
  out.push(...closing);
  return out;
}

function waitBeat(): Promise<void> {
  return new Promise((resolve) => {
    beatTimer = window.setTimeout(resolve, BEAT_MS);
  });
}

async function runExercise(ex: Exercise): Promise<void> {
  setMove(ex.move);
  skipMove = false;
  if (settingsVoice) speak(ex.speakStart);

  const totalBeats = ex.sets * BEATS_PER_SET;
  for (let beat = 0; beat < totalBeats; beat += 1) {
    if (stopAll || skipMove) break;
    const setIndex = Math.floor(beat / BEATS_PER_SET);
    const beatInSet = beat % BEATS_PER_SET;
    showExercise(ex, setIndex, beatInSet);
    if (settingsBeat) playBeat(beatInSet === 0);
    if (settingsVoice && beatInSet === 0 && beat > 0 && ex.cues[setIndex]) {
      speak(ex.cues[setIndex]);
    }
    await waitBeat();
  }
}

async function runRest(snap: TimerSnapshot) {
  const id = ++runId;
  stopAll = true;
  skipMove = true;
  window.clearTimeout(beatTimer);
  await new Promise((r) => window.setTimeout(r, 30));
  if (id !== runId) return;

  stopAll = false;
  skipMove = false;
  settingsVoice = snap.settings.voice;
  settingsBeat = snap.settings.beat;
  settingsStrict = snap.settings.strict;
  restDeadline = Date.now() + snap.remainingSecs * 1000;
  actionsEl.classList.toggle("hidden", settingsStrict);
  strictNote.hidden = !settingsStrict;
  fillSummary(summaryEl, snap.settings.summary ? snap.restSummary : null);
  await unlockAudio();

  const list = planRoutine(snap.remainingSecs * 1000);
  for (const ex of list) {
    if (stopAll || id !== runId) break;
    if (Date.now() + 2500 >= restDeadline) break;
    await runExercise(ex);
  }
  if (!stopAll && id === runId) setMove("idle");
}

function syncClock(remaining: number) {
  remainEl.textContent = formatClock(remaining);
}

document.querySelector("#skipMove")?.addEventListener("click", () => {
  skipMove = true;
});

document.querySelector("#endRest")?.addEventListener("click", async () => {
  if (settingsStrict) return;
  stopAll = true;
  skipMove = true;
  stopSpeak();
  await invoke("skip_rest");
});

window.addEventListener("keydown", async (e) => {
  if (e.code === "Escape" && !settingsStrict) {
    stopAll = true;
    skipMove = true;
    stopSpeak();
    await invoke("skip_rest");
  }
});

await listen<TimerSnapshot>("timer-tick", (event) => {
  if (event.payload.phase === "rest") {
    syncClock(event.payload.remainingSecs);
  }
});

await listen<TimerSnapshot>("rest-begin", async (event) => {
  syncClock(event.payload.remainingSecs);
  await runRest(event.payload);
});

await listen<TimerSnapshot>("rest-end", () => {
  stopAll = true;
  skipMove = true;
  stopSpeak();
  setMove("idle");
  window.clearTimeout(beatTimer);
  fillSummary(summaryEl, null);
});

const initial = await invoke<TimerSnapshot>("get_state");
if (initial.phase === "rest") {
  syncClock(initial.remainingSecs);
  await runRest(initial);
} else {
  setMove("idle");
  showExercise(ROUTINE[0], 0, 0);
}
