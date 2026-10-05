let sampleLine = "";

export function setImagePixelSample(line: string) {
  sampleLine = line;
}

export function getImagePixelSample(): string {
  return sampleLine;
}

export function clearImagePixelSample() {
  sampleLine = "";
}
