import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { beginGeminiBrowser, finishGeminiBrowser, cancelGeminiBrowser, connectGeminiApps, connectGeminiAppsFromBrowser, listDetectedBrowsers, listGeminiBrowserProfiles, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";
import { GeminiAppsSection } from "./GeminiAppsSection";

vi.mock("../../../../lib/tauri", () => ({
  connectGeminiApps: vi.fn(), disconnectGeminiApps: vi.fn(), getManualCookies: vi.fn(),
  beginGeminiBrowser: vi.fn(), finishGeminiBrowser: vi.fn(), cancelGeminiBrowser: vi.fn(),
  connectGeminiAppsFromBrowser: vi.fn(), listDetectedBrowsers: vi.fn(), listGeminiBrowserProfiles: vi.fn(),
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
    vi.mocked(listDetectedBrowsers).mockResolvedValue([]);
    vi.mocked(listGeminiBrowserProfiles).mockResolvedValue([]);
    vi.mocked(connectGeminiAppsFromBrowser).mockResolvedValue(undefined);
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
  it("imports only the browser and profile explicitly selected by the user", async () => {
    vi.mocked(listDetectedBrowsers).mockResolvedValue([
      { browserType: "edge", displayName: "Edge", profileCount: 1 },
      { browserType: "vivaldi", displayName: "Vivaldi", profileCount: 2 },
    ]);
    vi.mocked(listGeminiBrowserProfiles).mockResolvedValue(["Default", "Profile 1"]);
    const onChanged = section();
    const profile = await screen.findByRole("combobox", { name: "GeminiAppsProfile" });
    await screen.findByRole("option", { name: "Profile 1" });
    expect(screen.getByRole("combobox", { name: "SelectBrowser" })).toHaveValue("vivaldi");
    expect(connectGeminiAppsFromBrowser).not.toHaveBeenCalled();
    fireEvent.change(profile, { target: { value: "Profile 1" } });
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "verified-browser" }]);
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserImport")); });
    expect(connectGeminiAppsFromBrowser).toHaveBeenCalledExactlyOnceWith("vivaldi", "Profile 1");
    expect(connectGeminiApps).not.toHaveBeenCalled();
    expect(onChanged).toHaveBeenCalledOnce();
  });
  it("keeps the previous selection when browser verification fails", async () => {
    vi.mocked(listDetectedBrowsers).mockResolvedValue([{ browserType: "vivaldi", displayName: "Vivaldi", profileCount: 1 }]);
    vi.mocked(listGeminiBrowserProfiles).mockResolvedValue(["Default"]);
    vi.mocked(getManualCookies).mockResolvedValue([{ providerId: "geminiapps", provider: "Gemini Apps", savedAt: "previous" }]);
    vi.mocked(connectGeminiAppsFromBrowser).mockRejectedValue("Cookie verification failed");
    const onChanged = section();
    await screen.findByRole("option", { name: "Default" });
    await act(async () => { fireEvent.click(screen.getByText("GeminiAppsBrowserImport")); });
    expect(screen.getByRole("alert")).toHaveTextContent("Cookie verification failed");
    expect(screen.getByText(/previous/)).toBeInTheDocument();
    expect(onChanged).not.toHaveBeenCalled();
  });
  it("cannot import a stale profile while a different browser is loading", async () => {
    vi.mocked(listDetectedBrowsers).mockResolvedValue([
      { browserType: "vivaldi", displayName: "Vivaldi", profileCount: 1 },
      { browserType: "edge", displayName: "Edge", profileCount: 1 },
    ]);
    let resolveEdge: ((profiles: string[]) => void) | undefined;
    const edgeProfiles = new Promise<string[]>(resolve => { resolveEdge = resolve; });
    vi.mocked(listGeminiBrowserProfiles).mockImplementation(browser => browser === "edge" ? edgeProfiles : Promise.resolve(["Default"]));
    section();
    await screen.findByRole("option", { name: "Default" });
    fireEvent.change(screen.getByRole("combobox", { name: "SelectBrowser" }), { target: { value: "edge" } });
    expect(screen.getByText("GeminiAppsBrowserImport")).toBeDisabled();
    fireEvent.click(screen.getByText("GeminiAppsBrowserImport"));
    expect(connectGeminiAppsFromBrowser).not.toHaveBeenCalled();
    await act(async () => { resolveEdge?.(["Edge profile"]); });
    expect(screen.getByRole("combobox", { name: "GeminiAppsProfile" })).toHaveValue("Edge profile");
    expect(screen.getByText("GeminiAppsBrowserImport")).toBeEnabled();
  });
});
