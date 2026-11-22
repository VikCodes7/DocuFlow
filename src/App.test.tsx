import { mockIPC } from "@tauri-apps/api/mocks";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

// UI tests run against mocked IPC, so they work on both OSes without the
// native app (doc §11).
describe("App", () => {
  it("shows the version from Rust and switches screens", async () => {
    mockIPC((cmd) => {
      if (cmd === "app_info") return { version: "0.1.0" };
      if (cmd === "list_history") return [];
      return null;
    });

    render(<App />);

    expect(await screen.findByText("Version 0.1.0")).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Image tools" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "How your files stay local" }));
    expect(await screen.findByRole("heading", { name: "How your files stay local" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Recent jobs" }));
    expect(await screen.findByText("No jobs yet.")).toBeInTheDocument();
  });

  it("shows translated messages for backend error codes", async () => {
    mockIPC((cmd) => {
      if (cmd === "app_info") return { version: "0.1.0" };
      if (cmd === "list_history") throw { code: "Internal" };
      return null;
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Recent jobs" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Something went wrong");
  });
});
