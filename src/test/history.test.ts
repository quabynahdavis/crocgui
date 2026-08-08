import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor, tick } from "@testing-library/svelte";

// Mock Tauri modules BEFORE importing the component
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

import { invoke } from "@tauri-apps/api/core";
import HistoryPage from "../routes/history/+page.svelte";

const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.clearAllMocks();
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === "get_transfer_history") {
      return Promise.resolve({ transfers: [] });
    }
    return Promise.resolve(undefined);
  });
});

afterEach(() => {
  cleanup();
});

function makeTransfer(overrides: Record<string, unknown> = {}) {
  return {
    id: "tx1",
    direction: "send" as const,
    status: "completed" as const,
    files: ["/tmp/file.txt"],
    code: "1234-ABCD",
    started_at: "1700000000",
    completed_at: "1700000100",
    error: null,
    pinned: false,
    ...overrides,
  };
}

describe("HistoryPage", () => {
  it("initially shows a loading spinner", () => {
    const { container } = render(HistoryPage, {});
    expect(container.querySelector(".animate-spin")).toBeInTheDocument();
  });

  it("shows 'No transfers yet.' after loadHistory resolves with empty transfers", async () => {
    invokeMock.mockResolvedValueOnce({ transfers: [] });
    const { findByText } = render(HistoryPage, {});
    expect(await findByText("No transfers yet.")).toBeInTheDocument();
  });

  it("renders transfer cards with correct status text after loadHistory resolves", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [
            makeTransfer({ id: "tx1", status: "completed" }),
            makeTransfer({ id: "tx2", status: "in_progress" }),
          ],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByText, getByText } = render(HistoryPage, {});
    expect(await findByText("completed")).toBeInTheDocument();
    expect(getByText("in progress")).toBeInTheDocument();
  });

  it("shows Sent and Received tabs with correct counts", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [
            makeTransfer({ id: "tx1", direction: "send" }),
            makeTransfer({ id: "tx2", direction: "send" }),
            makeTransfer({ id: "tx3", direction: "receive" }),
          ],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByText } = render(HistoryPage, {});
    expect(await findByText("Sent")).toBeInTheDocument();
    expect(await findByText("Received")).toBeInTheDocument();
    expect(await findByText("2")).toBeInTheDocument();
    expect(await findByText("1")).toBeInTheDocument();
  });

  it("clicking the pin button calls invoke('set_record_pinned', ...)", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1", pinned: false })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByRole, getByRole } = render(HistoryPage, {});
    expect(await findByRole("button", { name: /pin/i })).toBeInTheDocument();
    const pinButton = getByRole("button", { name: /pin/i });
    await fireEvent.click(pinButton);

    expect(invokeMock).toHaveBeenCalledWith("set_record_pinned", { id: "tx1", pinned: true });
  });

  it("clicking 'Clear All' calls invoke('clear_transfer_history')", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1" })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByText } = render(HistoryPage, {});
    const clearButton = await findByText("Clear All");
    await fireEvent.click(clearButton);

    expect(invokeMock).toHaveBeenCalledWith("clear_transfer_history");
  });

  it("clicking the delete button opens the delete confirmation modal", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1" })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByRole, getByText } = render(HistoryPage, {});
    const deleteButton = await findByRole("button", { name: /delete/i });
    await fireEvent.click(deleteButton);

    expect(getByText("Delete transfer record?")).toBeInTheDocument();
  });

  it("in the delete modal, clicking 'Delete record only' calls invoke('delete_transfer_record', ...)", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1" })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByRole, findByText, getByText } = render(HistoryPage, {});
    const deleteButton = await findByRole("button", { name: /delete/i });
    await fireEvent.click(deleteButton);

    expect(await findByText("Delete transfer record?")).toBeInTheDocument();
    const deleteRecordButton = getByText("Delete record only");
    await fireEvent.click(deleteRecordButton);

    expect(invokeMock).toHaveBeenCalledWith("delete_transfer_record", { id: "tx1" });
    expect(invokeMock).not.toHaveBeenCalledWith("delete_record_files", expect.anything());
  });

  it("for a sent record with files, 'Also delete source files' calls invoke('delete_record_files', ...) then invoke('delete_transfer_record', ...)", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1", direction: "send", files: ["/tmp/file.txt"] })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByRole, findByText, getByText } = render(HistoryPage, {});
    const deleteButton = await findByRole("button", { name: /delete/i });
    await fireEvent.click(deleteButton);

    expect(await findByText("Delete transfer record?")).toBeInTheDocument();
    const alsoDeleteFilesButton = getByText("Also delete source files");
    await fireEvent.click(alsoDeleteFilesButton);

    expect(invokeMock).toHaveBeenCalledWith("delete_record_files", { id: "tx1" });
    expect(invokeMock).toHaveBeenCalledWith("delete_transfer_record", { id: "tx1" });
  });

  it("clicking Cancel in the delete modal closes it without invoking delete", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_transfer_history") {
        return Promise.resolve({
          transfers: [makeTransfer({ id: "tx1" })],
        });
      }
      return Promise.resolve(undefined);
    });

    const { findByRole, findByText, getByText, queryByText } = render(HistoryPage, {});
    const deleteButton = await findByRole("button", { name: /delete/i });
    await fireEvent.click(deleteButton);

    expect(await findByText("Delete transfer record?")).toBeInTheDocument();
    const cancelButton = getByText("Cancel");
    await fireEvent.click(cancelButton);

    expect(queryByText("Delete transfer record?")).not.toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith("delete_transfer_record", expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith("delete_record_files", expect.anything());
  });
});
