let ctx: AudioContext | null = null;

function audio(): AudioContext {
  if (!ctx) {
    ctx = new AudioContext();
  }
  return ctx;
}

export async function unlockAudio(): Promise<void> {
  const c = audio();
  if (c.state === "suspended") {
    await c.resume();
  }
}

function tone(
  freq: number,
  duration: number,
  gain: number,
  type: OscillatorType = "triangle",
) {
  const c = audio();
  if (c.state === "suspended") void c.resume();
  const t = c.currentTime;
  const osc = c.createOscillator();
  const g = c.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t);
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(gain, t + 0.012);
  g.gain.exponentialRampToValueAtTime(0.0001, t + duration);
  osc.connect(g).connect(c.destination);
  osc.start(t);
  osc.stop(t + duration + 0.02);
}

export function playBeat(accent: boolean): void {
  if (accent) {
    tone(784, 0.11, 0.1, "triangle");
  } else {
    tone(523.25, 0.08, 0.06, "sine");
  }
}

export function playBell(kind: "start" | "end"): void {
  if (kind === "start") {
    tone(523.25, 0.18, 0.09);
    setTimeout(() => tone(659.25, 0.22, 0.1), 140);
    setTimeout(() => tone(783.99, 0.35, 0.11), 280);
  } else {
    tone(783.99, 0.18, 0.09);
    setTimeout(() => tone(659.25, 0.22, 0.1), 150);
    setTimeout(() => tone(523.25, 0.4, 0.11), 300);
  }
}

function pickZhVoice(): SpeechSynthesisVoice | null {
  const voices = window.speechSynthesis.getVoices();
  return (
    voices.find((v) => /zh-CN/i.test(v.lang) && /Xiaoxiao|Huihui|Yaoyao/i.test(v.name)) ||
    voices.find((v) => /zh-CN/i.test(v.lang)) ||
    voices.find((v) => /^zh/i.test(v.lang)) ||
    null
  );
}

export function speak(text: string): void {
  if (!("speechSynthesis" in window) || !text) return;
  window.speechSynthesis.cancel();
  const u = new SpeechSynthesisUtterance(text);
  u.lang = "zh-CN";
  u.rate = 0.94;
  u.pitch = 1.04;
  const voice = pickZhVoice();
  if (voice) u.voice = voice;
  window.speechSynthesis.speak(u);
}

export function stopSpeak(): void {
  if ("speechSynthesis" in window) {
    window.speechSynthesis.cancel();
  }
}

if (typeof window !== "undefined" && "speechSynthesis" in window) {
  window.speechSynthesis.addEventListener("voiceschanged", () => {
    pickZhVoice();
  });
}
