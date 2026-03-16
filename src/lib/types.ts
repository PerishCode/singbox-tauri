export type RuntimePaths = {
  mode: "envOverride" | "dev" | "production";
  root: string;
  binDir: string;
  configDir: string;
  logsDir: string;
  stateDir: string;
  secretsDir: string;
  subscriptionsDir: string;
};

export type SingboxCheck = {
  name: string;
  ok: boolean;
  detail: string;
};

export type SingboxBootstrapReport = {
  binaryPath: string;
  logPath: string;
  pidPath: string;
  sessionPath: string;
  processStatus: string;
  version: string | null;
  checks: SingboxCheck[];
};
