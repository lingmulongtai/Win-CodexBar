import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { beginGeminiBrowser, finishGeminiBrowser, cancelGeminiBrowser, connectGeminiApps, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";
import { GeminiAppsSection } from "./GeminiAppsSection";

vi.mock("../../../../lib/tauri", () => ({
  connectGeminiApps: vi.fn(), disconnectGeminiApps: vi.fn(), getManualCookies: vi.fn(),
  beginGeminiBrowser: vi.fn(), finishGeminiBrowser: vi.fn(), cancelGeminiBrowser: vi.fn(),
}));
function section(disabled = false) {
  const onChanged = vi.fn();
  render(<GeminiAppsSection disabled={disabled} t={key => key} onChanged={onChanged} />);
  return onChanged;
}
describe("Gemini Apps connection", () => {
  beforeEach(() => {
    vi.clearAllMocks(); vi.mocked(getManualCookies).mockResolvedValue([]);
    vi.mocked(connectGeminiApps).mockResolvedValue(undefined);
    vi.mocked(disconnectGeminiApps).mockResolvedValue(undefined);
    vi.mocked(beginGeminiBrowser).mockResolvedValue(undefined);
    vi.mocked(finishGeminiBrowser).mockResolvedValue(undefined);
    vi.mocked(cancelGeminiBrowser).mockResolvedValue(undefined);
  });
  it("opens login without declaring connected, then commits only after verification", async () => {
    const onChanged = section();
    await screen.findByText("GeminiAppsNotConnected");
    expect(beginGeminiBrowser).not.toHaveBeenCalled();
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserOpen")); });
    expect(beginGeminiBrowser).toHaveBeenCalledOnce();
    expect(onChanged).not.toHaveBeenCalled();
    expect(screen.getByText("GeminiAppsNotConnected")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("GeminiAppsBrowserPending");
    expect(screen.queryByRole("textbox")).toBeNull();
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "verified" }]);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserFinish")); });
    expect(finishGeminiBrowser).toHaveBeenCalledOnce();
    expect(onChanged).toHaveBeenCalledOnce();
    expect(screen.getByText(/verified/)).toBeInTheDocument();
    expect(screen.queryByText("GeminiAppsBrowserFinish")).toBeNull();
  });
  it("keeps the previous connection and login retry visible when verification fails", async () => {
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "previous" }]);
    vi.mocked(finishGeminiBrowser).mockRejectedValue("Sign in first");
    const onChanged = section();
    await screen.findByText(/previous/);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserOpen")); });
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserFinish")); });
    expect(screen.getByRole("alert")).toHaveTextContent("Sign in first");
    expect(screen.getByText(/previous/)).toBeInTheDocument();
    expect(screen.getByText("GeminiAppsBrowserFinish")).toBeEnabled();
    expect(onChanged).not.toHaveBeenCalled();
  });
  it("cancels a pending login without disconnecting the previous account", async () => {
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "previous" }]);
    const onChanged = section();
    await screen.findByText(/previous/);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserOpen")); });
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserCancel")); });
    expect(cancelGeminiBrowser).toHaveBeenCalledOnce();
    expect(disconnectGeminiApps).not.toHaveBeenCalled();
    expect(onChanged).not.toHaveBeenCalled();
    expect(screen.queryByText("GeminiAppsBrowserFinish")).toBeNull();
    expect(screen.getByText(/previous/)).toBeInTheDocument();
  });
  it("imports only on an explicit click and shows no cookie input", async () => {
    const onChanged = section();
    await screen.findByText("GeminiAppsNotConnected");
    expect(connectGeminiApps).not.toHaveBeenCalled();
    expect(screen.queryByRole("textbox")).toBeNull();
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "2026-10-15 00:00" }]);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsConnect")); });
    expect(connectGeminiApps).toHaveBeenCalledOnce();
    expect(onChanged).toHaveBeenCalledOnce();
    expect(screen.getByText(/2026-10-15/)).toBeInTheDocument();
    expect(screen.getByText("GeminiAppsDisconnect")).toBeEnabled();
  });
  it("disconnects the Apps session without touching CLI credentials", async () => {
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "saved" }]);
    const onChanged = section();
    await screen.findByText(/GeminiAppsSaved/);
    vi.mocked(getManualCookies).mockResolvedValue([]);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsDisconnect")); });
    expect(disconnectGeminiApps).toHaveBeenCalledOnce();
    expect(onChanged).toHaveBeenCalledOnce();
    expect(screen.getByText("GeminiAppsNotConnected")).toBeInTheDocument();
  });
  it("keeps failed connection visible and disables actions during settings saves", async () => {
    vi.mocked(connectGeminiApps).mockRejectedValue("Close Gemini and reconnect");
    const onChanged = section(); await screen.findByText("GeminiAppsNotConnected");
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsConnect")); });
    expect(screen.getByRole("alert")).toHaveTextContent("Close Gemini and reconnect");
    expect(onChanged).not.toHaveBeenCalled();
  });
  it("shows revocation even when browser cleanup still needs retrying", async () => {
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "saved" }]);
    const onChanged = section();
    await screen.findByText(/GeminiAppsSaved/);
    vi.mocked(getManualCookies).mockResolvedValue([]);
    vi.mocked(disconnectGeminiApps).mockRejectedValue("Browser data is still busy");
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsDisconnect")); });
    expect(screen.getByText("GeminiAppsNotConnected")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Browser data is still busy");
    expect(onChanged).toHaveBeenCalledOnce();
  });
});
