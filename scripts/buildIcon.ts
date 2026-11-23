import { execSync } from "node:child_process";
import { copyFileSync } from "node:fs";

(() => {
  if (process.platform === "win32") {
    execSync(
      "powershell -NoProfile -ExecutionPolicy Bypass -File scripts/buildBrandIcon.ps1",
      { stdio: "inherit" },
    );
  }

  execSync("tauri icon src-tauri/assets/logo.png", { stdio: "inherit" });

  copyFileSync("src-tauri/icons/icon.png", "src-tauri/assets/logo.png");
  copyFileSync("src-tauri/icons/icon.png", "src-tauri/assets/logo-mac.png");
  copyFileSync("src-tauri/icons/icon.png", "public/logo.png");
  copyFileSync("src-tauri/icons/icon.png", "public/logo-mac.png");

  if (process.platform === "win32") {
    execSync(
      "powershell -NoProfile -ExecutionPolicy Bypass -File scripts/buildInstallerAssets.ps1",
      { stdio: "inherit" },
    );
  }
})();
