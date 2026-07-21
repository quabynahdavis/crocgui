class ReceiveState {
  code = $state("");
  outputDir = $state("");

  reset() {
    this.code = "";
    this.outputDir = "";
  }
}

export const receiveState = new ReceiveState();
