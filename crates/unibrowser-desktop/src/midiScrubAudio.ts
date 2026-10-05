let sharedCtx: AudioContext | null = null;

function ctx(): AudioContext {
  if (!sharedCtx) {
    sharedCtx = new AudioContext();
  }
  return sharedCtx;
}

function midiNoteToHz(note: number): number {
  return 440 * 2 ** ((note - 69) / 12);
}

/** Short audition beep for timeline MIDI scrub (opt-in preference). */
export function playMidiScrubBeeps(notes: { note: number; velocity: number }[]) {
  if (!notes.length) {
    return;
  }
  const sorted = [...notes].sort((a, b) => a.note - b.note);
  const pick = sorted[Math.floor(sorted.length / 2)];
  const audio = ctx();
  if (audio.state === "suspended") {
    void audio.resume();
  }
  const osc = audio.createOscillator();
  const gain = audio.createGain();
  const vel = Math.min(1, pick.velocity / 127);
  gain.gain.value = 0.04 + vel * 0.08;
  osc.frequency.value = midiNoteToHz(pick.note);
  osc.type = "triangle";
  osc.connect(gain);
  gain.connect(audio.destination);
  const now = audio.currentTime;
  osc.start(now);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.12);
  osc.stop(now + 0.14);
}
