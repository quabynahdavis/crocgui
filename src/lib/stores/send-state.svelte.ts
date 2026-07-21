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

  reset() {
    this.items = [];
    this.textInput = "";
    this.clipboardPasted = false;
    this.clipboardContent = "";
  }
}

export const sendState = new SendState();
