import { isToolbarOnlyWindow } from "./toolbarOnly";

if (isToolbarOnlyWindow()) {
  void import("./toolbarApp.ts");
} else {
  void import("./main.ts");
}
