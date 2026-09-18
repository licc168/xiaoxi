export type MoveId =
  | "idle"
  | "breathe"
  | "neck-lr"
  | "neck-ud"
  | "neck-roll"
  | "shrug"
  | "shoulder-roll"
  | "chest-open"
  | "twist"
  | "side-bend"
  | "reach-up"
  | "wrists"
  | "eyes"
  | "shake";

export interface Exercise {
  id: string;
  name: string;
  section: string;
  instruction: string;
  speakStart: string;
  cues: string[];
  move: MoveId;
  sets: number;
  closing?: boolean;
}

export const ROUTINE: Exercise[] = [
  {
    id: "breathe",
    name: "深呼吸",
    section: "准备活动",
    instruction: "双臂从体侧缓缓上举至头上，吸气；落下时慢慢呼气，肩膀放松。",
    speakStart: "准备活动，深呼吸。双臂慢慢举起，吸气。落下，呼气。",
    cues: ["吸气，举手", "呼气，落下", "再吸一次", "慢慢呼气"],
    move: "breathe",
    sets: 4,
  },
  {
    id: "neck-lr",
    name: "左右转头",
    section: "颈部运动",
    instruction: "一拍向左转头，停一拍；回正。再向右转头，停一拍，回正。动作要慢。",
    speakStart: "颈部运动。向左转头，停两拍。回正。向右转头。",
    cues: ["向左看", "回正，向右看", "再来一次，向左", "向右，回正"],
    move: "neck-lr",
    sets: 4,
  },
  {
    id: "neck-ud",
    name: "低头仰头",
    section: "颈部运动",
    instruction: "轻轻低头看脚尖，停两拍；再慢慢仰头看天花板，停两拍。不要猛甩。",
    speakStart: "低头看脚尖，停一停。再慢慢仰头。",
    cues: ["低头", "仰头", "再低头", "再仰头"],
    move: "neck-ud",
    sets: 4,
  },
  {
    id: "neck-roll",
    name: "头部环绕",
    section: "颈部运动",
    instruction: "头部沿圆周缓慢环绕，一圈八拍。幅度小一些，感到晕就减小动作。",
    speakStart: "头部慢慢环绕，跟着画圈，不要急。",
    cues: ["顺时针慢转", "再转一圈", "换个方向", "慢慢转回来"],
    move: "neck-roll",
    sets: 4,
  },
  {
    id: "shrug",
    name: "耸肩放松",
    section: "肩部运动",
    instruction: "双肩同时向上耸起，贴近耳朵，停两拍；再彻底放下，体会放松。",
    speakStart: "肩部运动。耸肩，停一停，再放下。",
    cues: ["耸肩", "放下", "再耸肩", "彻底放松"],
    move: "shrug",
    sets: 4,
  },
  {
    id: "shoulder-roll",
    name: "肩部绕环",
    section: "肩部运动",
    instruction: "双肩由前向后缓慢绕环，再由后向前。打开胸口，远离键盘。",
    speakStart: "双肩由前向后绕环。打开胸口。",
    cues: ["向后绕", "继续向后", "换向前绕", "再绕一圈"],
    move: "shoulder-roll",
    sets: 4,
  },
  {
    id: "chest-open",
    name: "扩胸运动",
    section: "扩胸运动",
    instruction: "双臂胸前平屈，向两侧打开后振两次，再慢慢合拢。抬头挺胸。",
    speakStart: "扩胸运动。双臂打开，扩胸。再向前合拢。",
    cues: ["打开扩胸", "向前合拢", "再打开", "挺胸抬头"],
    move: "chest-open",
    sets: 4,
  },
  {
    id: "twist",
    name: "体转运动",
    section: "腰部运动",
    instruction: "两脚分开站稳，双臂侧平举。上体转到一侧停两拍，再转另一侧。髋和脚不要跟着转。",
    speakStart: "体转运动。向左转体，再向右转体。脚不要动。",
    cues: ["向左转", "向右转", "再向左", "再向右"],
    move: "twist",
    sets: 4,
  },
  {
    id: "side-bend",
    name: "体侧运动",
    section: "腰部运动",
    instruction: "一手叉腰，另一臂上举，上体向侧弯。左右交替，呼吸均匀。",
    speakStart: "体侧运动。左手叉腰，右手上举，向左侧弯。",
    cues: ["向左弯", "向右弯", "再向左", "再向右"],
    move: "side-bend",
    sets: 4,
  },
  {
    id: "reach-up",
    name: "举手伸展",
    section: "上肢运动",
    instruction: "交替将一只手臂尽量向上伸，脚跟微微提起，把身体拉长。",
    speakStart: "举手伸展。左手尽量向上，换右手。把身体拉长。",
    cues: ["左手向上", "右手向上", "再拉长一点", "换手伸展"],
    move: "reach-up",
    sets: 4,
  },
  {
    id: "wrists",
    name: "手腕绕环",
    section: "腕部运动",
    instruction: "双臂稍稍抬起，只转手腕画圈，先向外再向内。打字久了特别需要。",
    speakStart: "手腕绕环。握空心拳，慢慢转手腕。",
    cues: ["向外转", "继续转", "换向内转", "再转一圈"],
    move: "wrists",
    sets: 4,
  },
  {
    id: "eyes",
    name: "远近调节",
    section: "眼保健操",
    instruction: "先看远处六米外的一个点，再看自己的指尖。眨眨眼，让眼睛休息。",
    speakStart: "眼保健操。先看远处，再看指尖。眨眨眼。",
    cues: ["看远处", "看指尖", "再看远处", "轻轻眨眼"],
    move: "eyes",
    sets: 4,
  },
  {
    id: "shake",
    name: "抖动放松",
    section: "整理运动",
    instruction: "双手自然下垂，轻轻抖动手腕和肩膀，脚也跟着弹一弹。课间操结束。",
    speakStart: "整理运动。抖动双手，放松肩膀。很好，休息一下。",
    cues: ["抖动手腕", "放松肩膀", "再抖一抖", "深呼一口气"],
    move: "shake",
    sets: 2,
    closing: true,
  },
];

export const BEAT_MS = 560;
export const BEATS_PER_SET = 8;

export function formatClock(secs: number): string {
  const s = Math.max(0, Math.floor(secs));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}`;
}

export function phraseMs(): number {
  return BEATS_PER_SET * BEAT_MS;
}

export function exerciseDurationMs(ex: Exercise): number {
  return ex.sets * phraseMs();
}
