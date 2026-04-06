import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

interface FilesChangedEvent {
  folder: string;
}

/**
 * Listen for file system changes in the sync directory.
 * Calls `onChanged` when files in any of the specified folders change.
 */
export function useFileWatcher(folders: string[], onChanged: () => void) {
  useEffect(() => {
    const unlisten = listen<FilesChangedEvent>("files-changed", ({ payload }) => {
      if (folders.some((f) => payload.folder.startsWith(f))) {
        onChanged();
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [folders, onChanged]);
}
