import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { connectGeminiApps, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";
import { GeminiAppsSection } from "./GeminiAppsSection";

vi.mock("../../../../lib/tauri", () => ({
  connectGeminiApps: vi.fn(), disconnectGeminiApps: vi.fn(), getManualCookies: vi.fn(),
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
});
