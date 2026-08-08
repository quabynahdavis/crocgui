import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";

// ---------------------------------------------------------------------------
// Tauri / SvelteKit mocks — declared before the component import so that
// `onMount` inside the receive page never touches a real Tauri IPC bridge.
// `vi.mock` is hoisted by vitest, hence the `vi.hoisted` handle below.
// ---------------------------------------------------------------------------
const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  unlisten: vi.fn(),
  open: vi.fn(),
  loadSettings: vi.fn(),
  searchParamsGet: vi.fn(),
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

vi.mock("$lib/settings", () => ({
  loadSettings: mocks.loadSettings,
}));

// `page` is a Svelte store; the receive page reads
// `$page.url.searchParams.get("code")` inside onMount.
vi.mock("$app/stores", async () => {
  const { writable: w } = await import("svelte/store");
  return {
    page: w({ url: { searchParams: { get: mocks.searchParamsGet } } }),
  };
});

// ---------------------------------------------------------------------------

import ReceivePage from "../routes/receive/+page.svelte";
import { receiveState } from "../lib/stores/receive-state.svelte";

const VALID_CODE = "1234-ABCD-5678-EFGH";

const SETTINGS = {
  relay: null,
  curve: "p256",
  disableCompression: false,
  outputDir: null,
};

/** Render and let onMount's awaited promises settle. */
async function renderPage() {
  const utils = render(ReceivePage, {});
  await waitFor(() => expect(mocks.listen).toHaveBeenCalled());
  await tick();
  return utils;
}

function getCodeInput(container: HTMLElement): HTMLInputElement {
  return container.querySelector("#code") as HTMLInputElement;
}

function getReceiveButton(): HTMLButtonElement {
  return [...document.querySelectorAll("button")].find((b) =>
    b.textContent?.includes("Receive Files"),
  ) as HTMLButtonElement;
}

describe("Receive page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    receiveState.reset();

    mocks.invoke.mockResolvedValue(undefined);
    mocks.listen.mockResolvedValue(mocks.unlisten);
    mocks.open.mockResolvedValue(null);
    mocks.loadSettings.mockResolvedValue(SETTINGS);
    mocks.searchParamsGet.mockReturnValue(null);
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the Receive Files card with a code input and a Receive Files button", async () => {
    const { getByText, getAllByText, getByLabelText, container } = await renderPage();

    // "Receive Files" appears twice: the card title and the submit button.
    expect(getAllByText("Receive Files")).toHaveLength(2);
    expect(
      getByText("Enter the code phrase from the sender to receive their file"),
    ).toBeInTheDocument();
    expect(getByLabelText("Code Phrase")).toBeInTheDocument();
    expect(getCodeInput(container)).toBeInTheDocument();
    expect(getReceiveButton()).toBeInTheDocument();
  });

  it("disables the Receive button initially (empty code)", async () => {
    await renderPage();
    expect(getReceiveButton()).toBeDisabled();
  });

  it("enables the Receive button when a valid code is typed", async () => {
    const { container } = await renderPage();

    await fireEvent.input(getCodeInput(container), { target: { value: VALID_CODE } });
    await tick();

    expect(receiveState.code).toBe(VALID_CODE);
    expect(getReceiveButton()).not.toBeDisabled();
  });

  it("keeps the button disabled and shows the error hint for an invalid code", async () => {
    const { container, getByText } = await renderPage();

    await fireEvent.input(getCodeInput(container), { target: { value: "nope" } });
    await tick();

    expect(getReceiveButton()).toBeDisabled();
    expect(getByText("Code must look like 1234-ABCD-5678-EFGH")).toBeInTheDocument();
  });

  it("hides the error hint once the code becomes valid", async () => {
    const { container, queryByText } = await renderPage();

    await fireEvent.input(getCodeInput(container), { target: { value: "nope" } });
    await tick();
    expect(queryByText("Code must look like 1234-ABCD-5678-EFGH")).toBeInTheDocument();

    await fireEvent.input(getCodeInput(container), { target: { value: VALID_CODE } });
    await tick();
    expect(queryByText("Code must look like 1234-ABCD-5678-EFGH")).not.toBeInTheDocument();
  });

  it("calls invoke('receive_file') with the code and sets transferring on click", async () => {
    const { container } = await renderPage();

    await fireEvent.input(getCodeInput(container), { target: { value: VALID_CODE } });
    await tick();
    await fireEvent.click(getReceiveButton());
    await tick();

    expect(mocks.invoke).toHaveBeenCalledWith(
      "receive_file",
      expect.objectContaining({ code: VALID_CODE }),
    );
    expect(receiveState.transferring).toBe(true);
    expect(receiveState.progressPercent).toBe(0);
  });

  it("shows the progress bar and Cancel button while transferring", async () => {
    const { container, getByText, queryByRole } = await renderPage();

    await fireEvent.input(getCodeInput(container), { target: { value: VALID_CODE } });
    await tick();
    await fireEvent.click(getReceiveButton());
    await tick();

    expect(container.querySelector("progress")).toBeInTheDocument();
    expect(getByText("Cancel")).toBeInTheDocument();
    // the form section is swapped out while transferring
    expect(getCodeInput(container)).toBeNull();
  });

  it("calls invoke('cancel_transfer') and resets transferring when Cancel is clicked", async () => {
    receiveState.code = VALID_CODE;
    receiveState.transferring = true;
    const { getByText } = await renderPage();

    await fireEvent.click(getByText("Cancel"));
    await tick();

    expect(mocks.invoke).toHaveBeenCalledWith("cancel_transfer");
    await waitFor(() => expect(receiveState.transferring).toBe(false));
    expect(receiveState.status).toBe("cancelled");
  });

  it("shows the completion message and Receive Another File button when status is complete", async () => {
    receiveState.status = "complete";
    const { getByText, queryByText } = await renderPage();

    expect(getByText("Transfer complete! Files have been saved.")).toBeInTheDocument();
    expect(getByText("Receive Another File")).toBeInTheDocument();
    expect(queryByText("Code must look like 1234-ABCD-5678-EFGH")).not.toBeInTheDocument();
  });

  it("resets back to the form when Receive Another File is clicked", async () => {
    receiveState.status = "complete";
    receiveState.code = VALID_CODE;
    receiveState.progressLog = ["50%"];
    const { getByText, container } = await renderPage();

    await fireEvent.click(getByText("Receive Another File"));
    await tick();

    expect(receiveState.status).toBe("");
    expect(receiveState.code).toBe("");
    expect(receiveState.progressLog.length).toBe(0);
    expect(getCodeInput(container)).toBeInTheDocument();
  });

  it("auto-uppercases a lowercase code as it is typed", async () => {
    const { container } = await renderPage();
    const input = getCodeInput(container);

    await fireEvent.input(input, { target: { value: "1234-abcd-5678-efgh" } });
    await tick();

    expect(receiveState.code).toBe(VALID_CODE);
    expect(input.value).toBe(VALID_CODE);
  });

  it("populates and uppercases receiveState.code from the ?code= query param on mount", async () => {
    mocks.searchParamsGet.mockReturnValue("abcd-1234-efgh-5678");

    const { container } = await renderPage();

    expect(mocks.searchParamsGet).toHaveBeenCalledWith("code");
    await waitFor(() => expect(receiveState.code).toBe("ABCD-1234-EFGH-5678"));
    await tick();
    expect(getCodeInput(container).value).toBe("ABCD-1234-EFGH-5678");
    expect(getReceiveButton()).not.toBeDisabled();
  });

  it("does not overwrite an existing code with the ?code= query param", async () => {
    receiveState.code = VALID_CODE;
    mocks.searchParamsGet.mockReturnValue("zzzz-9999-yyyy-8888");

    await renderPage();

    expect(receiveState.code).toBe(VALID_CODE);
  });

  it("registers croc-progress, croc-receive-complete and croc-error listeners on mount", async () => {
    await renderPage();

    await waitFor(() => expect(mocks.listen).toHaveBeenCalledTimes(3));
    const events = mocks.listen.mock.calls.map((c) => c[0]);
    expect(events).toEqual(
      expect.arrayContaining(["croc-progress", "croc-receive-complete", "croc-error"]),
    );
  });
});
