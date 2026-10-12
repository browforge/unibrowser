const stub = {
  label: "main",
  async setTitle(_title: string) {},
  async minimize() {},
  async maximize() {},
  async close() {},
  async isMaximized() {
    return false;
  },
};

export function getCurrentWindow() {
  return stub;
}
