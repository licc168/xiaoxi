export type Phase = "idle" | "work" | "rest";

export interface Settings {
  workMinutes: number;
  restMinutes: number;
  voice: boolean;
  beat: boolean;
  strict: boolean;
  summary: boolean;
}

export interface AppRow {
  name: string;
  title: string;
  secs: number;
}

export interface WorkSummary {
  workSecs: number;
  idleSecs: number;
  keystrokes: number;
  apps: AppRow[];
}

export interface TimerSnapshot {
  phase: Phase;
  remainingSecs: number;
  totalSecs: number;
  paused: boolean;
  settings: Settings;
  lastSummary: WorkSummary | null;
  restSummary: WorkSummary | null;
}
