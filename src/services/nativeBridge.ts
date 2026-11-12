import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  appName: string;
  version: string;
  operatingSystem: string;
  architecture: string;
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}


export async function consumeStartupSplash(): Promise<boolean> {
  return invoke<boolean>("consume_startup_splash");
}
