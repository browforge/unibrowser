import type { DocumentViewerDto } from "./documentViewer";
import type { PackageStorageSessionDto } from "./packageStorageUi";

type CaptureTabState = {
  tabs: Array<{
    id: number;
    label: string;
    location_kind: string;
    is_directory: boolean;
    is_http: boolean;
    dirty: boolean;
    pinned: boolean;
    muted: boolean;
    mute_available: boolean;
    file_path?: string | null;
  }>;
  active_index: number;
  omnibox: string;
  show_new_tab_page: boolean;
  document_viewer: DocumentViewerDto | null;
  text_document: null;
  package_storage: PackageStorageSessionDto | null;
};

/** Doc-capture: show archive tab + project surface without blocking package open IPC. */
export async function applyPkgArchiveDocCaptureUi(archivePath: string): Promise<void> {
  const apply = (
    window as unknown as { __selfTestApplyState?: (d: unknown) => Promise<void> }
  ).__selfTestApplyState;
  if (!apply) {
    return;
  }
  const label = archivePath.split(/[/\\]/).pop() ?? "pkg-capture.unibrow";
  const viewer: DocumentViewerDto = {
    path: archivePath,
    viewer: "unibrow",
    title: label.replace(/\.unibrow$/i, "") || "Demo",
    project_nodes: [
      {
        id: "doc-1",
        node_type: "document",
        label: "Field guide",
        asset: "document.md",
        x: 40,
        y: 48,
      },
      {
        id: "img-1",
        node_type: "image",
        label: "Logo",
        asset: "logo.svg",
        x: 200,
        y: 48,
      },
    ],
    project_clips: [],
  };
  const packageStorage: PackageStorageSessionDto = {
    archive_path: archivePath,
    bundle_root: archivePath,
    mode: "archive",
    history_path: archivePath,
    edited: false,
    history_notice_dismissed: true,
    unpacked_bytes: 0,
    suggest_folder: false,
    unpack_in_progress: false,
    unpack_progress: 0,
  };
  const patch: CaptureTabState = {
    tabs: [
      {
        id: 1,
        label,
        location_kind: "file",
        is_directory: false,
        is_http: false,
        dirty: false,
        pinned: false,
        muted: false,
        mute_available: false,
        file_path: archivePath,
      },
    ],
    active_index: 0,
    omnibox: archivePath,
    show_new_tab_page: false,
    document_viewer: viewer,
    text_document: null,
    package_storage: packageStorage,
  };
  const base: Record<string, unknown> = {
    mods_loaded: "",
    status: "",
    directory_entries: null,
    document_status: null,
    mod_portal: null,
    unknown_file_banner: null,
    show_onboarding: false,
    show_onboarding_tour: false,
    recent_files: [],
    split_view_enabled: false,
    split_index: null,
    split_text_document: null,
    split_document_viewer: null,
    split_status: null,
    browse_folder_path: "",
    diff_view: null,
    pinned_locations: [],
    ntp_sections: {
      show_hero: true,
      show_actions: true,
      show_browse: true,
      show_recent: true,
    },
    files_view: { sort: "name", sort_ascending: true, show_hidden: false },
    folder_favorites: [],
  };
  await apply({ ...base, ...patch });
}
