export type ProviderStatus = "available" | "limited" | "planned-disabled";

export interface TwokeyRuntimeStatus {
  waylandGlobalHotkeys: ProviderStatus;
  textToSpeech: ProviderStatus;
  trayMenu: ProviderStatus;
  sqliteHistoryAudit: ProviderStatus;
  onlineProviders: ProviderStatus;
}

export interface TwokeyPackageInfo {
  name: string;
  repository: string;
  runtimeStatus: TwokeyRuntimeStatus;
}

export const runtimeStatus: TwokeyRuntimeStatus = {
  waylandGlobalHotkeys: "limited",
  textToSpeech: "available",
  trayMenu: "available",
  sqliteHistoryAudit: "available",
  onlineProviders: "available",
};

export const packageInfo: TwokeyPackageInfo = {
  name: "twokey",
  repository: "https://github.com/meinzeug/twokey",
  runtimeStatus,
};

export function getPackageInfo(): TwokeyPackageInfo {
  return packageInfo;
}
