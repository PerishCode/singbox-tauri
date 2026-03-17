import {
  getSingboxTauri,
  type ControlSnapshotResponse,
  type ControlStateResponse,
  type SingboxRuntimeStatus,
} from "./generated";
import { localApiRequest } from "./http";
import type { LocalNetworkSnapshot } from "../types";

const api = getSingboxTauri();

export async function fetchControlState(): Promise<ControlStateResponse> {
  return api.state();
}

export async function fetchControlSnapshot(): Promise<ControlSnapshotResponse> {
  return api.snapshot();
}

export async function appendAppEvent(message: string): Promise<void> {
  await api.appendEvent({ message });
}

export async function startSingbox(): Promise<SingboxRuntimeStatus> {
  return api.start();
}

export async function stopSingbox(): Promise<SingboxRuntimeStatus> {
  return api.stop();
}

export async function restartSingbox(): Promise<SingboxRuntimeStatus> {
  return api.restart();
}

export async function fetchAppLog(): Promise<string> {
  return api.appLog();
}

export async function fetchSingboxLog(): Promise<string> {
  return api.singboxLog();
}

export async function fetchLocalNetworkSnapshot(): Promise<LocalNetworkSnapshot> {
  const response = await localApiRequest<{ network: LocalNetworkSnapshot }>({
    url: "/api/v1/network",
    method: "GET",
  });
  return response.network;
}
