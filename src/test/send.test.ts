import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";

// ---------------------------------------------------------------------------
// Tauri / plugin mocks — must be declared before the component import so that
// `onMount` inside the send page never touches a real Tauri IPC bridge.
// `vi.mock` is hoisted by vitest, hence the `vi.hoisted` handles below.
// ---------------------------------------------------------------------------
const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  unlisten: vi.fn(),
  open: vi.fn(),
  readText: vi.fn(),
  writeText: vi.fn(),
  loadSettings: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: mocks.open,
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: mocks.readText,
  writeText: mocks.writeText,
}));

vi.mock("$lib/settings", () => ({
  loadSettings: mocks.loadSettings,
}));

import SendPage from "../routes/send/+page.svelte";
import { sendState } from "../lib/stores/send-state.svelte";

const SETTINGS = {
  relay: null,
  curve: "p256",
  disableCompression: false,
  outputDir: null,
};

describe("send page", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.listen.mockResolvedValue(mocks.unlisten);
    mocks.loadSettings.mockResolvedValue(SETTINGS);
    mocks.open.mockResolvedValue(null);
    mocks.readText.mockResolvedValue("");
    mocks.writeText.mockResolvedValue(undefined);
    mocks.invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "save_temp_text") return "/tmp/croc/note.txt";
      return undefined;
    });

    // Singleton store — hard reset between tests.
    sendState.reset();
    sendState.mode = "file";
  });

  afterEach(() => {
    cleanup();
    sendState.reset();
    sendState.mode = "file";
  });

  // 1 ------------------------------------------------------------------
  it("renders the Send Files card with all mode selector buttons", () => {
    const { getByText } = render(SendPage, {});

    expect(getByText("Send Files")).toBeInTheDocument();
    expect(getByText("Select files, a folder, or share text")).toBeInTheDocument();

    for (const label of ["File", "Folder", "Text", "Paste"]) {
      expect(getByText(label)).toBeInTheDocument();
    }
  });

  it("registers croc event listeners on mount", async () => {
    render(SendPage, {});

    // The four listeners are registered sequentially with `await`, so poll
    // until the mount microtask chain has drained.
    await waitFor(() => {
      const events = mocks.listen.mock.calls.map((c) => c[0]);
      expect(events).toEqual(
        expect.arrayContaining(["croc-progress", "croc-code", "croc-complete", "croc-error"]),
      );
    });
  });

  // 2 ------------------------------------------------------------------
  it("clicking a mode button updates sendState.mode", async () => {
    const { getByText } = render(SendPage, {});

    await fireEvent.click(getByText("Folder"));
    expect(sendState.mode).toBe("folder");

    await fireEvent.click(getByText("Text"));
    expect(sendState.mode).toBe("text");

    await fireEvent.click(getByText("Paste"));
    expect(sendState.mode).toBe("clipboard");

    await fireEvent.click(getByText("File"));
    expect(sendState.mode).toBe("file");
  });

  it("switching to text mode reveals the textarea", async () => {
    const { getByText, getByPlaceholderText, queryByPlaceholderText } = render(SendPage, {});

    expect(queryByPlaceholderText("Type or paste your text here…")).toBeNull();

    await fireEvent.click(getByText("Text"));
    await tick();

    expect(getByPlaceholderText("Type or paste your text here…")).toBeInTheDocument();
  });

  // 3 ------------------------------------------------------------------
  it("shows the send button and item count once items exist", async () => {
    sendState.items = [
      { type: "file", path: "/tmp/a.txt", label: "a.txt" },
      { type: "file", path: "/tmp/b.txt", label: "b.txt" },
    ];

    const { getByText } = render(SendPage, {});

    expect(getByText("2 item(s) selected")).toBeInTheDocument();
    expect(getByText(/Send\s+Items/)).toBeInTheDocument();
  });

  it("uses the singular label for a single item", () => {
    sendState.items = [{ type: "file", path: "/tmp/a.txt", label: "a.txt" }];

    const { getByText } = render(SendPage, {});

    expect(getByText("1 item(s) selected")).toBeInTheDocument();
    expect(getByText(/Send\s+Item/)).toBeInTheDocument();
  });

  // 4 ------------------------------------------------------------------
  it("clicking send marks the transfer as running and invokes send_file", async () => {
    sendState.items = [{ type: "file", path: "/tmp/a.txt", label: "a.txt" }];

    const { getByText } = render(SendPage, {});

    await fireEvent.click(getByText(/Send\s+Item/));

    await waitFor(() => {
      expect(mocks.invoke).toHaveBeenCalledWith("send_file", {
        paths: ["/tmp/a.txt"],
        ...SETTINGS,
      });
    });

    expect(sendState.transferring).toBe(true);
    expect(sendState.status).toBe("starting");
    expect(mocks.loadSettings).toHaveBeenCalled();
  });

  it("persists text items to a temp file before sending", async () => {
    sendState.items = [{ type: "text", path: "", label: "note-1.txt", preview: "hello world" }];

    const { getByText } = render(SendPage, {});

    await fireEvent.click(getByText(/Send\s+Item/));

    await waitFor(() => {
      expect(mocks.invoke).toHaveBeenCalledWith("save_temp_text", {
        filename: "note-1.txt",
        content: "hello world",
      });
    });

    await waitFor(() => {
      expect(mocks.invoke).toHaveBeenCalledWith("send_file", {
        paths: ["/tmp/croc/note.txt"],
        ...SETTINGS,
      });
    });
  });

  it("surfaces an error and stops transferring when send_file rejects", async () => {
    mocks.invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "send_file") throw new Error("boom");
      return undefined;
    });
    sendState.items = [{ type: "file", path: "/tmp/a.txt", label: "a.txt" }];

    const { getByText } = render(SendPage, {});
    await fireEvent.click(getByText(/Send\s+Item/));

    await waitFor(() => {
      expect(sendState.transferring).toBe(false);
      expect(sendState.status).toContain("error");
    });
  });

  // 5 ------------------------------------------------------------------
  it("shows a progress bar and cancel button while transferring", async () => {
    sendState.transferring = true;
    sendState.progressPercent = 42;

    const { getByText, container } = render(SendPage, {});

    const bar = container.querySelector("progress");
    expect(bar).not.toBeNull();
    expect(bar).toHaveValue(42);
    expect(getByText("Cancel")).toBeInTheDocument();
  });

  it("hides the mode selector while transferring", () => {
    sendState.transferring = true;

    const { queryByText } = render(SendPage, {});

    expect(queryByText("Folder")).toBeNull();
    expect(queryByText("Paste")).toBeNull();
  });

  // 6 ------------------------------------------------------------------
  it("clicking cancel invokes cancel_transfer and clears transferring", async () => {
    sendState.transferring = true;

    const { getByText } = render(SendPage, {});

    await fireEvent.click(getByText("Cancel"));

    await waitFor(() => {
      expect(mocks.invoke).toHaveBeenCalledWith("cancel_transfer");
    });

    await waitFor(() => {
      expect(sendState.transferring).toBe(false);
      expect(sendState.status).toBe("cancelled");
    });
  });

  // 7 ------------------------------------------------------------------
  it("lists every item label with a remove button", () => {
    sendState.items = [
      { type: "file", path: "/tmp/a.txt", label: "a.txt" },
      { type: "folder", path: "/tmp/dir", label: "dir" },
      { type: "text", path: "", label: "note-1.txt", preview: "hi" },
    ];

    const { getByText, getAllByLabelText } = render(SendPage, {});

    expect(getByText("a.txt")).toBeInTheDocument();
    expect(getByText("dir")).toBeInTheDocument();
    expect(getByText("note-1.txt")).toBeInTheDocument();
    expect(getAllByLabelText(/remove/i)).toHaveLength(3);
  });

  it("removing an item drops it from the store and the list", async () => {
    sendState.items = [
      { type: "file", path: "/tmp/a.txt", label: "a.txt" },
      { type: "file", path: "/tmp/b.txt", label: "b.txt" },
    ];

    const { getAllByLabelText, queryByText } = render(SendPage, {});

    await fireEvent.click(getAllByLabelText(/remove/i)[0]);
    await tick();

    expect(sendState.items.map((i) => i.label)).toEqual(["b.txt"]);
    expect(queryByText("a.txt")).toBeNull();
  });

  // 8 ------------------------------------------------------------------
  it("resets the store when 'Send Another Item' is clicked after completion", async () => {
    sendState.items = [{ type: "file", path: "/tmp/a.txt", label: "a.txt" }];
    sendState.status = "complete";
    sendState.code = "1-apple-banana";
    sendState.progressPercent = 100;
    sendState.progressLog = ["100%"];

    const { getByText, queryByText } = render(SendPage, {});

    expect(getByText("Transfer complete!")).toBeInTheDocument();

    const resetSpy = vi.spyOn(sendState, "reset");
    await fireEvent.click(getByText("Send Another Item"));
    await tick();

    expect(resetSpy).toHaveBeenCalledTimes(1);
    expect(sendState.items).toEqual([]);
    expect(sendState.status).toBe("");
    expect(sendState.code).toBe("");
    expect(sendState.progressPercent).toBe(0);
    expect(queryByText("Transfer complete!")).toBeNull();

    resetSpy.mockRestore();
  });

  // 9 ------------------------------------------------------------------
  it("adds a text item from the textarea", async () => {
    sendState.mode = "text";

    const { getByText, getByPlaceholderText } = render(SendPage, {});

    const textarea = getByPlaceholderText("Type or paste your text here…");
    await fireEvent.input(textarea, { target: { value: "some shared note" } });
    await tick();

    expect(sendState.textInput).toBe("some shared note");

    await fireEvent.click(getByText("Add as note.txt"));
    await tick();

    expect(sendState.items).toHaveLength(1);
    expect(sendState.items[0].type).toBe("text");
    expect(sendState.items[0].preview).toBe("some shared note");
    expect(sendState.items[0].label).toMatch(/^note-\d+\.txt$/);
    expect(sendState.textInput).toBe("");
  });

  it("ignores blank text input", async () => {
    sendState.mode = "text";

    const { getByText, getByPlaceholderText } = render(SendPage, {});

    const textarea = getByPlaceholderText("Type or paste your text here…");
    await fireEvent.input(textarea, { target: { value: "   " } });
    await tick();

    await fireEvent.click(getByText("Add as note.txt"));
    await tick();

    expect(sendState.items).toHaveLength(0);
  });

  // 10 -----------------------------------------------------------------
  it("renders the share code when sendState.code is set", () => {
    sendState.code = "4-mango-signal-tunnel";

    const { getByText } = render(SendPage, {});

    expect(getByText("Share this code with the recipient:")).toBeInTheDocument();
    expect(getByText("4-mango-signal-tunnel")).toBeInTheDocument();
  });

  it("copies the code to the clipboard when the code is clicked", async () => {
    sendState.code = "4-mango-signal-tunnel";

    const { getByText } = render(SendPage, {});

    await fireEvent.click(getByText("4-mango-signal-tunnel"));

    await waitFor(() => {
      expect(mocks.writeText).toHaveBeenCalledWith("4-mango-signal-tunnel");
    });

    expect(await waitFor(() => getByText("Copied!"))).toBeInTheDocument();
  });

  it("does not render the code block when no code is set", () => {
    const { queryByText } = render(SendPage, {});

    expect(queryByText("Share this code with the recipient:")).toBeNull();
  });
});
