export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size_bytes?: number;
  modified_unix?: number;
  is_image?: boolean;
  is_archive?: boolean;
}

export interface FilesViewDto {
  sort: string;
  sort_ascending: boolean;
  show_hidden: boolean;
}
