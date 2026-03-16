import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type RuntimePaths = {
  mode: "envOverride" | "dev" | "production";
  root: string;
  binDir: string;
  configDir: string;
  logsDir: string;
  stateDir: string;
  secretsDir: string;
  subscriptionsDir: string;
};

function App() {
  const [runtimePaths, setRuntimePaths] = useState<RuntimePaths | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void invoke<RuntimePaths>("get_runtime_paths")
      .then((paths) => {
        setRuntimePaths(paths);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      });
  }, []);

  const entries = runtimePaths
    ? [
        ["mode", runtimePaths.mode],
        ["root", runtimePaths.root],
        ["bin", runtimePaths.binDir],
        ["config", runtimePaths.configDir],
        ["logs", runtimePaths.logsDir],
        ["state", runtimePaths.stateDir],
        ["secrets", runtimePaths.secretsDir],
        ["subscriptions", runtimePaths.subscriptionsDir],
      ]
    : [];

  return (
    <main className="container">
      <section className="panel">
        <p className="eyebrow">singbox-tauri</p>
        <h1>Runtime Paths</h1>
        <p className="description">
          Unified runtime root for development and production.
        </p>
        {error ? <p className="error">{error}</p> : null}
        <div className="grid">
          {entries.map(([label, value]) => (
            <div key={label} className="item">
              <span className="label">{label}</span>
              <code>{value}</code>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}

export default App;
