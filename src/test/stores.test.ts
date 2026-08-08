import { describe, it, expect, beforeEach } from "vitest";
import { sendState } from "../lib/stores/send-state.svelte";
import { receiveState } from "../lib/stores/receive-state.svelte";

describe("sendState", () => {
  beforeEach(() => {
    sendState.reset();
  });

  it("starts with empty items", () => {
    expect(sendState.items.length).toBe(0);
  });

  it("starts in file mode", () => {
    expect(sendState.mode).toBe("file");
  });

  it("tracks transferring state", () => {
    expect(sendState.transferring).toBe(false);
    sendState.transferring = true;
    expect(sendState.transferring).toBe(true);
  });

  it("tracks code", () => {
    expect(sendState.code).toBe("");
    sendState.code = "1234-ABCD";
    expect(sendState.code).toBe("1234-ABCD");
  });

  it("tracks status", () => {
    expect(sendState.status).toBe("");
    sendState.status = "starting";
    expect(sendState.status).toBe("starting");
  });

  it("tracks progress log", () => {
    expect(sendState.progressLog.length).toBe(0);
    sendState.progressLog = ["line1", "line2"];
    expect(sendState.progressLog.length).toBe(2);
  });

  it("reset clears all state", () => {
    sendState.items = [{ type: "file", path: "/tmp/test.txt", label: "test.txt" }];
    sendState.code = "code";
    sendState.transferring = true;
    sendState.status = "complete";
    sendState.progressLog = ["line"];
    sendState.textInput = "hello";
    sendState.clipboardPasted = true;
    sendState.clipboardContent = "clipboard data";

    sendState.reset();

    expect(sendState.items.length).toBe(0);
    expect(sendState.code).toBe("");
    expect(sendState.transferring).toBe(false);
    expect(sendState.status).toBe("");
    expect(sendState.progressLog.length).toBe(0);
    expect(sendState.textInput).toBe("");
    expect(sendState.clipboardPasted).toBe(false);
    expect(sendState.clipboardContent).toBe("");
  });

  it("clearing clipboard content on reset prevents memory leak", () => {
    sendState.clipboardContent = "sensitive clipboard data";
    sendState.reset();
    expect(sendState.clipboardContent).toBe("");
  });
});

describe("receiveState", () => {
  beforeEach(() => {
    receiveState.reset();
  });

  it("starts with empty code", () => {
    expect(receiveState.code).toBe("");
  });

  it("starts with empty output dir", () => {
    expect(receiveState.outputDir).toBe("");
  });

  it("tracks transferring state", () => {
    expect(receiveState.transferring).toBe(false);
    receiveState.transferring = true;
    expect(receiveState.transferring).toBe(true);
  });

  it("reset clears all state", () => {
    receiveState.code = "1234-ABCD";
    receiveState.outputDir = "/tmp";
    receiveState.transferring = true;
    receiveState.status = "complete";
    receiveState.progressLog = ["line"];

    receiveState.reset();

    expect(receiveState.code).toBe("");
    expect(receiveState.outputDir).toBe("");
    expect(receiveState.transferring).toBe(false);
    expect(receiveState.status).toBe("");
    expect(receiveState.progressLog.length).toBe(0);
  });
});
