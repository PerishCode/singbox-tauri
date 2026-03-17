import axios, { type AxiosRequestConfig } from "axios";

export const localApi = axios.create({
  baseURL: "http://127.0.0.1:18427",
});

export async function localApiRequest<T>(config: AxiosRequestConfig): Promise<T> {
  const response = await localApi.request<T>(config);
  return response.data;
}
