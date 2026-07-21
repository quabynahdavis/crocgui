export type SendMode = "file" | "folder" | "text" | "clipboard";

export interface SendItem {
  type: SendMode;
  path: string;
  label: string;
  preview?: string;
}

class SendState {
  items = $state<SendItem[]>([]);
  mode = $state<SendMode>("file");
  textInput = $state("");
  clipboardPasted = $state(false);
  clipboardContent = $state("");

  transferring = $state(false);
  code = $state("");
  status = $state("");
  progressLog = $state<string[]>([]);

  reset() {
    this.items = [];
    this.textInput = "";
    this.clipboardPasted = false;
    this.clipboardContent = "";
    this.transferring = false;
    this.code = "";
    this.status = "";
    this.progressLog = [];
  }
}

export const sendState = new SendState();
