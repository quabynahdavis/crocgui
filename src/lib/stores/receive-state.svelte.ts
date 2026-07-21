class ReceiveState {
  code = $state("");
  outputDir = $state("");

  transferring = $state(false);
  status = $state("");
  progressLog = $state<string[]>([]);
  progressPercent = $state(0);

  reset() {
    this.code = "";
    this.outputDir = "";
    this.transferring = false;
    this.status = "";
    this.progressLog = [];
    this.progressPercent = 0;
  }
}

export const receiveState = new ReceiveState();
