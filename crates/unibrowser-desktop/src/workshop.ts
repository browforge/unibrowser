import { invoke } from "@tauri-apps/api/core";
import { showPickList, type PickListItem } from "./inlineUi";
import { showToast } from "./toast";

interface WorkshopNode {
  id: string;
  node_type: string;
  label: string;
  asset: string | null;
}

interface WorkshopClip {
  id: string;
  node_id: string;
  node_type: string;
  start_sec: number;
  duration_sec: number;
  label: string;
  asset?: string | null;
  title_text?: string | null;
  pdf_page?: number | null;
}

interface WorkshopMarker {
  id: string;
  time_sec: number;
  title: string;
}

export interface WorkshopViewer {
  path: string;
  title: string;
  project_nodes?: WorkshopNode[];
  project_clips?: WorkshopClip[];
  project_markers?: WorkshopMarker[];
}

type WorkshopMode = "preview" | "live" | "recording";
type TrackKind = "visual" | "title" | "audio";
let desiredWorkshopMode: WorkshopMode = "preview";

function trackIndexForClip(clip: WorkshopClip): number {
  const kind = trackForClip(clip);
  if (kind === "visual") {
    return 1;
  }
  if (kind === "title") {
    return 2;
  }
  return 3;
}

function trackForClip(clip: WorkshopClip): TrackKind {
  const type = clip.node_type.toLowerCase();
  if (type.includes("title") || type.includes("text")) {
    return "title";
  }
  if (type.includes("audio") || type.includes("midi")) {
    return "audio";
  }
  return "visual";
}

function typeAbbrev(type: string): string {
  const value = type.toLowerCase();
  if (value.includes("title")) return "TTL";
  if (value.includes("midi")) return "MID";
  if (value.includes("audio")) return "AUD";
  if (value.includes("pdf")) return "PDF";
  if (value.includes("video")) return "VID";
  return "IMG";
}

function refreshProject(data: unknown) {
  window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
}

export async function renderWorkshop(viewer: WorkshopViewer, root: HTMLElement) {
  root.replaceChildren();
  root.className = "workshop";

  const nodes = viewer.project_nodes ?? [];
  const clips = viewer.project_clips ?? [];
  const markers = viewer.project_markers ?? [];
  const totalSec = Math.max(
    10,
    Math.ceil(clips.reduce((end, clip) => Math.max(end, clip.start_sec + clip.duration_sec), 0)),
  );

  const defaultTitleClip =
    clips.find((clip) => clip.node_type.toLowerCase().includes("title")) ?? clips[0] ?? null;
  let selectedClip: WorkshopClip | null = defaultTitleClip;
  let selectedNode = selectedClip
    ? nodes.find((node) => node.id === selectedClip?.node_id) ?? null
    : nodes[0] ?? null;
  let mode: WorkshopMode = desiredWorkshopMode;
  let snap = true;
  let timelineZoom = 1;

  const sources = document.createElement("aside");
  sources.className = "workshop-sources";
  sources.setAttribute("aria-label", "Sources");
  const sourcesHeading = document.createElement("h2");
  sourcesHeading.className = "workshop-region-heading";
  sourcesHeading.textContent = "Sources";
  const sourcesList = document.createElement("div");
  sourcesList.className = "workshop-source-list";
  sources.append(sourcesHeading, sourcesList);

  const stageRegion = document.createElement("section");
  stageRegion.className = "workshop-stage-region";
  stageRegion.setAttribute("aria-label", "Stage");
  const stageHead = document.createElement("div");
  stageHead.className = "workshop-stage-head";
  const stageTitle = document.createElement("h2");
  stageTitle.className = "workshop-region-heading";
  stageTitle.textContent = "Stage";
  const state = document.createElement("span");
  state.className = "workshop-state";
  state.dataset.state = mode;
  state.textContent = mode === "preview" ? "Preview" : mode === "live" ? "LIVE" : "REC";
  stageHead.append(stageTitle, state);
  const stageWell = document.createElement("div");
  stageWell.className = "workshop-stage-well";
  const stageFrame = document.createElement("div");
  stageFrame.className = "workshop-stage-frame";
  const stageContent = document.createElement("div");
  stageContent.className = "workshop-stage-content";
  stageFrame.appendChild(stageContent);
  stageWell.appendChild(stageFrame);
  const transport = document.createElement("div");
  transport.className = "workshop-transport";
  transport.setAttribute("aria-label", "Programme transport");
  const transportPlay = document.createElement("button");
  transportPlay.type = "button";
  transportPlay.className = "workshop-transport-btn";
  transportPlay.textContent = "Play";
  transportPlay.setAttribute("aria-label", "Play programme");
  const transportTime = document.createElement("span");
  transportTime.className = "workshop-transport-time";
  transport.append(transportPlay, transportTime);
  stageRegion.append(stageHead, stageWell, transport);

  const inspector = document.createElement("aside");
  inspector.className = "workshop-inspector";
  inspector.setAttribute("aria-label", "Inspector");
  const inspectorHeading = document.createElement("h2");
  inspectorHeading.className = "workshop-region-heading";
  inspectorHeading.textContent = "Inspector";
  const inspectorBody = document.createElement("div");
  inspectorBody.className = "workshop-inspector-body";
  inspector.append(inspectorHeading, inspectorBody);

  const timeline = document.createElement("section");
  timeline.className = "workshop-timeline";
  timeline.setAttribute("aria-label", "Timeline");
  const timelineHead = document.createElement("div");
  timelineHead.className = "workshop-timeline-head";
  const timelineTitle = document.createElement("h2");
  timelineTitle.className = "workshop-region-heading";
  timelineTitle.textContent = "Timeline";
  const timelineStatus = document.createElement("span");
  timelineStatus.className = "workshop-timeline-status";
  const timelineTools = document.createElement("span");
  timelineTools.className = "workshop-timeline-tools";
  timelineHead.append(timelineTitle, timelineTools, timelineStatus);
  const timelineViewport = document.createElement("div");
  timelineViewport.className = "workshop-timeline-viewport";
  const timelineCanvas = document.createElement("div");
  timelineCanvas.className = "workshop-timeline-canvas";
  timelineViewport.appendChild(timelineCanvas);
  timeline.append(timelineHead, timelineViewport);

  root.append(sources, stageRegion, inspector, timeline);

  let playheadSec = 0;
  const syncTimelineChrome = () => {
    root.dataset.snap = snap ? "on" : "off";
    root.dataset.timelineZoom = String(timelineZoom);
    timelineTools.textContent = `Snap ${snap ? "on" : "off"} · Zoom ${Math.round(timelineZoom * 100)}%`;
    timelineStatus.textContent = formatTimecode(playheadSec);
    syncTransport();
    timelineCanvas.classList.toggle("workshop-timeline-canvas--snap", snap);
  };

  const formatTimecode = (sec: number) => {
    const whole = Math.max(0, Math.floor(sec));
    const mins = Math.floor(whole / 60);
    const secs = whole % 60;
    const frames = Math.floor((sec - whole) * 30);
    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}:${String(frames).padStart(2, "0")}`;
  };

  const syncTransport = () => {
    transportTime.textContent = formatTimecode(playheadSec);
    transportPlay.textContent = playheadSec > 0.05 ? "Pause" : "Play";
  };

  const renderStage = () => {
    stageContent.replaceChildren();
    syncTransport();
    if (mode !== "preview") {
      const live = document.createElement("div");
      live.className = "workshop-programme-card workshop-programme-card--live";
      const label = document.createElement("strong");
      label.textContent = mode === "recording" ? "Recording programme" : "Live programme";
      const detail = document.createElement("span");
      detail.textContent = "Local compositor";
      live.append(label, detail);
      stageContent.appendChild(live);
      return;
    }
    const current = selectedClip ?? clips[0];
    const frame = document.createElement("div");
    frame.className = "workshop-programme-frame";
    const programme = document.createElement("div");
    programme.className = "workshop-programme-video";
    const sceneLabel = document.createElement("span");
    sceneLabel.className = "workshop-programme-scene";
    sceneLabel.textContent = current?.label ?? viewer.title;
    programme.appendChild(sceneLabel);
    frame.appendChild(programme);
    const isTitle =
      current &&
      (current.node_type.toLowerCase().includes("title") ||
        trackForClip(current) === "title" ||
        Boolean(current.title_text));
    if (isTitle) {
      const lowerThird = document.createElement("div");
      lowerThird.className = "workshop-lower-third workshop-lower-third--selected";
      const heading = document.createElement("strong");
      heading.textContent = current?.title_text || current?.label || viewer.title;
      const subtitle = document.createElement("span");
      subtitle.textContent = current?.label ?? "Programme title";
      lowerThird.append(heading, subtitle);
      frame.appendChild(lowerThird);
    }
    stageContent.appendChild(frame);
  };

  const field = (
    label: string,
    value: string,
    type: "text" | "number" = "text",
  ): { row: HTMLLabelElement; input: HTMLInputElement } => {
    const row = document.createElement("label");
    row.className = "workshop-field";
    const caption = document.createElement("span");
    caption.textContent = label;
    const input = document.createElement("input");
    input.type = type;
    input.value = value;
    row.append(caption, input);
    return { row, input };
  };

  const renderInspector = () => {
    inspectorBody.replaceChildren();
    if (mode !== "preview") {
      const health = document.createElement("section");
      health.className = "workshop-inspector-section";
      health.innerHTML =
        '<h3>Source health</h3><div class="workshop-health"><span class="workshop-health-dot"></span><span>Programme source</span><strong>Ready</strong></div>';
      const mixer = document.createElement("section");
      mixer.className = "workshop-inspector-section";
      const mixerTitle = document.createElement("h3");
      mixerTitle.textContent = "Mixer";
      const level = document.createElement("div");
      level.className = "workshop-meter";
      level.innerHTML = "<span></span><span></span><span></span><span></span><span></span>";
      const mute = document.createElement("button");
      mute.type = "button";
      mute.className = "workshop-quiet-button";
      mute.textContent = "Mute programme";
      mixer.append(mixerTitle, level, mute);
      inspectorBody.append(health, mixer);
      return;
    }

    if (!selectedClip) {
      const empty = document.createElement("p");
      empty.className = "workshop-empty";
      empty.textContent = selectedNode
        ? `${selectedNode.label} is selected in Sources. Add it to the timeline to edit timing.`
        : "Select a source or clip.";
      inspectorBody.appendChild(empty);
      return;
    }

    const isTitleClip =
      selectedClip.node_type.toLowerCase().includes("title") ||
      trackForClip(selectedClip) === "title" ||
      Boolean(selectedClip.title_text);
    const selectionHeader = document.createElement("div");
    selectionHeader.className = "workshop-selection-header";
    const selectionTitle = document.createElement("p");
    selectionTitle.className = "workshop-selection-title";
    selectionTitle.textContent = isTitleClip ? "Lower third" : selectedClip.label;
    const selectionSubtitle = document.createElement("p");
    selectionSubtitle.className = "workshop-selection-meta";
    const trackLabel = isTitleClip ? "Title" : trackForClip(selectedClip) === "visual" ? "Video / Image" : "Audio / MIDI";
    selectionSubtitle.textContent = `${isTitleClip ? "Title" : trackLabel} clip, track ${trackIndexForClip(selectedClip)}`;
    selectionHeader.append(selectionTitle, selectionSubtitle);
    const clipLabel = field(isTitleClip ? "Heading" : "Clip label", selectedClip.label, "text");
    const typeRow = field("Type", selectedClip.node_type, "text");
    typeRow.input.readOnly = true;
    typeRow.input.classList.add("workshop-field-readonly");
    const start = field("Start (seconds)", selectedClip.start_sec.toFixed(1), "number");
    const duration = field(
      isTitleClip ? "Duration" : "Duration (seconds)",
      selectedClip.duration_sec.toFixed(1),
      "number",
    );
    start.input.min = "0";
    start.input.step = "0.1";
    duration.input.min = "0.1";
    duration.input.step = "0.1";
    const apply = document.createElement("button");
    apply.type = "button";
    apply.className = "workshop-primary-button";
    apply.textContent = "Apply timing";
    apply.onclick = async () => {
      const data = await invoke("update_unibrow_timeline_clip", {
        projectPath: viewer.path,
        clipId: selectedClip!.id,
        startSec: Number.parseFloat(start.input.value) || 0,
        durationSec: Math.max(0.1, Number.parseFloat(duration.input.value) || 0.1),
      });
      refreshProject(data);
    };
    const fields: HTMLElement[] = [selectionHeader];
    if (isTitleClip) {
      const heading = field("Heading", selectedClip.title_text ?? selectedClip.label, "text");
      const subtitle = field("Subtitle", selectedClip.label, "text");
      const position = field("Position", "Lower third", "text");
      position.input.readOnly = true;
      position.input.classList.add("workshop-field-readonly");
      fields.push(heading.row, subtitle.row, duration.row, position.row);
      heading.input.addEventListener("change", () => {
        void invoke("set_unibrow_timeline_clip_title_text", {
          projectPath: viewer.path,
          clipId: selectedClip!.id,
          titleText: heading.input.value,
        }).then((data) => refreshProject(data));
      });
      subtitle.input.addEventListener("change", () => {
        const next = subtitle.input.value.trim();
        if (!next) {
          return;
        }
        void invoke("rename_unibrow_timeline_clip", {
          projectPath: viewer.path,
          clipId: selectedClip!.id,
          label: next,
        }).then((data) => refreshProject(data));
      });
    } else {
      fields.push(clipLabel.row, typeRow.row, start.row, duration.row);
      clipLabel.input.addEventListener("change", () => {
        const next = clipLabel.input.value.trim();
        if (!next) {
          return;
        }
        void invoke("rename_unibrow_timeline_clip", {
          projectPath: viewer.path,
          clipId: selectedClip!.id,
          label: next,
        }).then((data) => refreshProject(data));
      });
    }
    if (
      !isTitleClip &&
      (selectedClip.node_type.toLowerCase().includes("pdf") ||
        (selectedClip.asset && selectedClip.asset.toLowerCase().endsWith(".pdf")))
    ) {
      const pdfPage = field("PDF page", String(selectedClip.pdf_page ?? 1), "number");
      pdfPage.input.min = "1";
      pdfPage.input.step = "1";
      pdfPage.input.addEventListener("change", () => {
        const page = Math.max(1, Number.parseInt(pdfPage.input.value, 10) || 1);
        void invoke("set_unibrow_timeline_clip_pdf_page", {
          projectPath: viewer.path,
          clipId: selectedClip!.id,
          page,
        }).then((data) => refreshProject(data));
      });
      fields.push(pdfPage.row);
    }
    fields.push(apply);
    inspectorBody.append(...fields);
  };

  const selectClip = (clip: WorkshopClip) => {
    selectedClip = clip;
    selectedNode = nodes.find((node) => node.id === clip.node_id) ?? null;
    timelineCanvas
      .querySelectorAll(".workshop-clip--selected")
      .forEach((element) => element.classList.remove("workshop-clip--selected"));
    timelineCanvas
      .querySelector<HTMLElement>(`[data-clip-id="${clip.id}"]`)
      ?.classList.add("workshop-clip--selected");
    renderInspector();
    renderStage();
  };

  const renderSources = () => {
    sourcesList.replaceChildren();
    nodes.forEach((node) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "workshop-source";
      button.dataset.nodeId = node.id;
      const badge = document.createElement("span");
      badge.className = "workshop-source-badge";
      badge.textContent = typeAbbrev(node.node_type);
      const copy = document.createElement("span");
      copy.className = "workshop-source-copy";
      const name = document.createElement("strong");
      name.textContent = node.label;
      const detail = document.createElement("span");
      detail.textContent = node.asset ?? node.node_type;
      copy.append(name, detail);
      button.append(badge, copy);
      button.onclick = () => {
        selectedNode = node;
        const firstClip = clips.find((clip) => clip.node_id === node.id);
        if (firstClip) {
          selectClip(firstClip);
          return;
        }
        selectedClip = null;
        renderInspector();
      };
      sourcesList.appendChild(button);
    });
  };

  const playhead = document.createElement("div");
  playhead.className = "workshop-playhead";
  const playheadCap = document.createElement("span");
  playhead.appendChild(playheadCap);

  const secFromPointer = (clientX: number, track: HTMLElement) => {
    const rect = track.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    return (ratio * totalSec) / timelineZoom;
  };

  const movePlayhead = (sec: number) => {
    playheadSec = Math.max(0, Math.min(totalSec, sec));
    playhead.style.left = `${(playheadSec / totalSec) * 100 * timelineZoom}%`;
    syncTimelineChrome();
  };

  const renderTimeline = () => {
    timelineCanvas.replaceChildren();
    timelineCanvas.style.setProperty("--timeline-zoom", String(timelineZoom));
    const ruler = document.createElement("div");
    ruler.className = "workshop-ruler";
    for (let second = 0; second <= totalSec; second += 1) {
      const tick = document.createElement("button");
      tick.type = "button";
      tick.className = "workshop-ruler-tick";
      tick.style.left = `${(second / totalSec) * 100}%`;
      tick.textContent = `${second}s`;
      tick.onclick = () => movePlayhead(second);
      ruler.appendChild(tick);
    }
    markers.forEach((marker) => {
      const pin = document.createElement("span");
      pin.className = "workshop-marker";
      pin.style.left = `${(marker.time_sec / totalSec) * 100 * timelineZoom}%`;
      pin.title = `${marker.title} @ ${marker.time_sec.toFixed(1)}s`;
      const tag = document.createElement("span");
      tag.className = "workshop-marker-tag";
      tag.textContent = marker.title;
      pin.appendChild(tag);
      ruler.appendChild(pin);
    });
    timelineCanvas.appendChild(ruler);

    const trackDefs: Array<[TrackKind, string]> = [
      ["visual", "Video / Image"],
      ["title", "Title"],
      ["audio", "Audio / MIDI"],
    ];
    trackDefs.forEach(([kind, label]) => {
      const row = document.createElement("div");
      row.className = "workshop-track-row";
      const trackLabel = document.createElement("div");
      trackLabel.className = "workshop-track-label";
      trackLabel.textContent = label;
      const lane = document.createElement("div");
      lane.className = "workshop-track-lane";
      lane.dataset.track = kind;
      lane.onclick = (event) => {
        if (event.target === lane) {
          movePlayhead(secFromPointer(event.clientX, lane));
        }
      };
      clips
        .filter((clip) => trackForClip(clip) === kind)
        .forEach((clip) => {
          const block = document.createElement("button");
          block.type = "button";
          block.className = `workshop-clip workshop-clip--${kind}`;
          block.dataset.clipId = clip.id;
          block.style.left = `${(clip.start_sec / totalSec) * 100}%`;
          block.style.width = `${Math.max(3, (clip.duration_sec / totalSec) * 100)}%`;
          const type = document.createElement("span");
          type.className = "workshop-clip-type";
          type.textContent = typeAbbrev(clip.node_type);
          const name = document.createElement("strong");
          name.textContent = clip.label;
          const duration = document.createElement("span");
          duration.className = "workshop-clip-duration";
          duration.textContent = `${clip.duration_sec.toFixed(1)}s`;
          block.append(type, name, duration);
          const startHandle = document.createElement("span");
          startHandle.className = "workshop-trim workshop-trim--start";
          const endHandle = document.createElement("span");
          endHandle.className = "workshop-trim workshop-trim--end";
          block.append(startHandle, endHandle);
          if (selectedClip?.id === clip.id) {
            block.classList.add("workshop-clip--selected");
          }

          const updateTiming = async (startSec: number, durationSec: number) => {
            const data = await invoke("update_unibrow_timeline_clip", {
              projectPath: viewer.path,
              clipId: clip.id,
              startSec: Math.max(0, startSec),
              durationSec: Math.max(0.1, durationSec),
            });
            refreshProject(data);
          };

          const bindTrim = (handle: HTMLElement, edge: "start" | "end") => {
            handle.onpointerdown = (event) => {
              event.preventDefault();
              event.stopPropagation();
              handle.setPointerCapture(event.pointerId);
              handle.onpointerup = (up) => {
                const at = secFromPointer(up.clientX, lane);
                if (edge === "start") {
                  const end = clip.start_sec + clip.duration_sec;
                  const start = Math.min(end - 0.1, Math.max(0, at));
                  void updateTiming(start, end - start);
                } else {
                  void updateTiming(clip.start_sec, Math.max(0.1, at - clip.start_sec));
                }
                handle.releasePointerCapture(up.pointerId);
              };
            };
          };
          bindTrim(startHandle, "start");
          bindTrim(endHandle, "end");

          block.onpointerdown = (event) => {
            if ((event.target as HTMLElement).classList.contains("workshop-trim")) {
              return;
            }
            selectClip(clip);
            const originX = event.clientX;
            block.setPointerCapture(event.pointerId);
            block.onpointerup = (up) => {
              const rect = lane.getBoundingClientRect();
              const delta = ((up.clientX - originX) / rect.width) * totalSec;
              let start = Math.max(0, clip.start_sec + delta);
              if (snap) start = Math.round(start * 2) / 2;
              block.releasePointerCapture(up.pointerId);
              void updateTiming(start, clip.duration_sec);
            };
          };
          lane.appendChild(block);
        });
      row.append(trackLabel, lane);
      timelineCanvas.appendChild(row);
    });
    timelineCanvas.appendChild(playhead);
    movePlayhead(playheadSec);
    syncTimelineChrome();
  };

  const applyModeUi = (next: WorkshopMode) => {
    desiredWorkshopMode = next;
    mode = next;
    state.dataset.state = next;
    state.textContent = next === "preview" ? "Preview" : next === "live" ? "LIVE" : "REC";
    root.dataset.mode = next;
    renderStage();
    renderInspector();
  };

  const setMode = async (next: WorkshopMode) => {
    applyModeUi(next);
    if (next === "preview") {
      await invoke("set_unibrow_live_mode", { enabled: false }).catch(() => undefined);
    } else {
      await invoke("set_unibrow_live_mode", { enabled: true }).catch(() => undefined);
      if (next === "recording") {
        await invoke("start_unibrow_live_recording").catch(() => undefined);
      }
    }
  };

  const visualMode = (event: Event) => {
    applyModeUi((event as CustomEvent<WorkshopMode>).detail);
  };

  const openExportDialog = async () => {
    const formats: PickListItem[] = [
      { id: "project", label: "Project package", detail: "Portable .unibrow archive" },
      { id: "share", label: "Share package", detail: "Project without derived output" },
      { id: "preview", label: "Preview MP4", detail: "Quick local preview" },
      { id: "h264", label: "H.264 MP4", detail: "Production video" },
    ];
    const picked = await showPickList("Export programme", formats);
    if (!picked) {
      return;
    }
    showToast(`${picked.label} export is not wired in this build.`, "info");
  };

  const projectCommand = async (event: Event) => {
    const command = (event as CustomEvent<string>).detail;
    if (command === "unibrowser.project.preview") return void setMode("preview");
    if (command === "unibrowser.project.live") return void setMode("live");
    if (command === "unibrowser.project.record") {
      return void setMode(mode === "recording" ? "live" : "recording");
    }
    if (command === "unibrowser.project.snap") {
      snap = !snap;
      syncTimelineChrome();
      showToast(`Timeline snap ${snap ? "on" : "off"}`, "info");
      return;
    }
    if (command === "unibrowser.project.zoomTimeline") {
      timelineZoom = timelineZoom >= 2 ? 1 : timelineZoom + 0.5;
      renderTimeline();
      showToast(`Timeline zoom ${Math.round(timelineZoom * 100)}%`, "info");
      return;
    }
    if (command === "unibrowser.project.deleteClip" && selectedClip) {
      const data = await invoke("remove_unibrow_timeline_clip", {
        projectPath: viewer.path,
        clipId: selectedClip.id,
      });
      refreshProject(data);
      return;
    }
    if (command === "unibrowser.project.addTitle") {
      const data = await invoke("add_unibrow_title_timeline_clip", {
        projectPath: viewer.path,
        titleText: "New title",
      });
      refreshProject(data);
      return;
    }
    if (command === "unibrowser.project.export") {
      await openExportDialog();
      return;
    }
    if (command === "unibrowser.project.marker") {
      const data = await invoke("add_unibrow_timeline_marker", {
        projectPath: viewer.path,
        timeSec: playheadSec,
        title: `Marker ${markers.length + 1}`,
      });
      refreshProject(data);
      showToast(`Marker at ${playheadSec.toFixed(1)}s`, "info");
      return;
    }
    showToast("Select a source or clip to use this project command.", "info");
  };

  window.addEventListener("unibrow-project-command", projectCommand, { signal: workshopAbort.signal });
  window.addEventListener("unibrow-workshop-visual-mode", visualMode, {
    signal: workshopAbort.signal,
  });
  renderSources();
  renderTimeline();
  if (defaultTitleClip) {
    selectClip(defaultTitleClip);
  } else {
    renderInspector();
    renderStage();
  }
  root.dataset.mode = mode;
}

let workshopAbort = new AbortController();

export function resetWorkshopListeners() {
  workshopAbort.abort();
  workshopAbort = new AbortController();
}
