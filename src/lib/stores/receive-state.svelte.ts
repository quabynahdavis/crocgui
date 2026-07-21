class ReceiveState {
  code = $state("");
  outputDir = $state("");

  transferring = $state(false);
  status = $state("");
  progressLog = $state<string[]>([]);

  reset() {
    this.code = "";
    this.outputDir = "";
    this.transferring = false;
    this.status = "";
    this.progressLog = [];
  }
}

export const receiveState = new ReceiveState();
