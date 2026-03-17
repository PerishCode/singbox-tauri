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

export type SingboxRuntimeStatus = {
  binaryPath: string;
  configPath: string;
  logPath: string;
  pidPath: string;
  sessionPath: string;
  processStatus: "stopped" | "starting" | "running" | "stopping";
  lifecycle:
    | "stopped"
    | "starting"
    | "runningPassive"
    | "runningSelective"
    | "runningFull"
    | "stopping";
  mode: "passive" | "selective" | "full";
  pid: number | null;
  version: string | null;
};

export type NetworkReadiness = "safe" | "caution" | "blocked" | "unknown";

export type NetworkConflictLevel = "info" | "warning" | "blocking";

export type NetworkProcessSignal = {
  pid: number;
  label: string;
  command: string;
};

export type NetworkPortBinding = {
  port: number;
  protocol: string;
  process: string;
  detail: string;
};

export type NetworkDefaultRoute = {
  interface: string | null;
  gateway: string | null;
};

export type NetworkInterfaceSummary = {
  name: string;
  kind: string;
  addresses: string[];
  isUp: boolean;
  isActive: boolean;
};

export type NetworkDnsResolver = {
  scope: string;
  resolvers: string[];
};

export type NetworkProxyStatus = {
  kind: string;
  enabled: boolean;
  host: string | null;
  port: number | null;
};

export type NetworkConflict = {
  level: NetworkConflictLevel;
  code: string;
  message: string;
  evidence: string[];
};

export type NetworkDiagnostics = {
  defaultRouteRaw: string;
  proxyRaw: string;
  dnsRaw: string;
  ifconfigRaw: string;
  listenRaw: string;
  processRaw: string;
};

export type LocalNetworkSnapshot = {
  readiness: NetworkReadiness;
  headline: string;
  reasons: string[];
  defaultRoute: NetworkDefaultRoute;
  interfaces: NetworkInterfaceSummary[];
  dnsResolvers: NetworkDnsResolver[];
  proxies: NetworkProxyStatus[];
  defaultInterface: string | null;
  defaultGateway: string | null;
  activeInterfaces: string[];
  utunInterfaces: string[];
  resolvers: string[];
  systemProxyEnabled: boolean;
  systemProxySummary: string[];
  relatedProcesses: NetworkProcessSignal[];
  portBindings: NetworkPortBinding[];
  conflicts: NetworkConflict[];
  diagnostics: NetworkDiagnostics;
};

export type SubscriptionKeyState = "missing" | "ready";

export type SubscriptionFetchState = "idle" | "fetched" | "failed";

export type SubscriptionDecryptState = "idle" | "ready" | "failed";

export type SubscriptionSnapshot = {
  keyState: SubscriptionKeyState;
  fetchState: SubscriptionFetchState;
  decryptState: SubscriptionDecryptState;
  sourceUrl: string | null;
  privateKeyPath: string;
  publicKeyPath: string;
  encryptedPath: string;
  decryptedPath: string;
  activeConfigPath: string;
  publicKey: string | null;
  lastError: string | null;
};
