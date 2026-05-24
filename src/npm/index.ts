export type ProviderStatus = "available" | "planned-disabled";

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
  waylandGlobalHotkeys: "planned-disabled",
  textToSpeech: "planned-disabled",
  trayMenu: "planned-disabled",
  sqliteHistoryAudit: "planned-disabled",
  onlineProviders: "planned-disabled",
};

export const packageInfo: TwokeyPackageInfo = {
  name: "twokey",
  repository: "https://github.com/meinzeug/twokey",
  runtimeStatus,
};

export function getPackageInfo(): TwokeyPackageInfo {
  return packageInfo;
}
