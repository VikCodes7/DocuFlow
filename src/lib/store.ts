import { create } from "zustand";

export type View = "imageTools" | "pdfTools" | "history" | "privacy";

interface AppState {
  view: View;
  setView: (view: View) => void;
}

/** Keep only references (paths, ids) and display metadata here, never file data. */
export const useAppStore = create<AppState>((set) => ({
  view: "imageTools",
  setView: (view) => set({ view }),
}));
