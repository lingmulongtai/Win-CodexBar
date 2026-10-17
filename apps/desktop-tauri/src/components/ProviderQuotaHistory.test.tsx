import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import ProviderQuotaHistory from "./ProviderQuotaHistory";
import { getProviderQuotaHistory } from "../lib/tauri";
import type { ProviderUsageSnapshot } from "../types/bridge";

vi.mock("../lib/tauri", () => ({ getProviderQuotaHistory: vi.fn() }));
const now = Date.parse("2026-10-18T00:00:00Z");
const second = now / 1000;
const provider = { providerId: "codex", updatedAt: "current" } as ProviderUsageSnapshot;
const t = (key: string) => key;
beforeEach(() => vi.resetAllMocks());

it("shows the archived month when 30 days is selected, without inventing missing values", async () => {
  vi.mocked(getProviderQuotaHistory).mockResolvedValue([{ label: "Week", samples: [
    { t: second - 15 * 86400, u: 0.2, r: null }, { t: second - 86400, u: 0.4, r: null },
  ] }]);
  const { container } = render(<ProviderQuotaHistory provider={provider} t={t} now={now} />);
  await screen.findByRole("img");
  expect(getProviderQuotaHistory).toHaveBeenCalledWith("codex", "current");
  expect(container.querySelectorAll("circle")).toHaveLength(1);
  fireEvent.click(screen.getByRole("button", { name: "QuotaArchiveThirtyDays" }));
  expect(container.querySelectorAll("circle")).toHaveLength(2);
  expect(screen.getByText("60% PanelLeftSuffix")).toBeInTheDocument();
});

it("ignores a late response after switching providers", async () => {
  let resolveOld: ((value: []) => void) | undefined;
  vi.mocked(getProviderQuotaHistory).mockImplementation(id => id === "codex"
    ? new Promise(resolve => { resolveOld = resolve; })
    : Promise.resolve([{ label: "New account", samples: [{ t: second, u: 0.1, r: null }] }]));
  const { rerender } = render(<ProviderQuotaHistory provider={provider} t={t} now={now} />);
  rerender(<ProviderQuotaHistory provider={{ ...provider, providerId: "claude", updatedAt: "replacement" }} t={t} now={now} />);
  await screen.findByText("New account");
  await act(async () => { resolveOld?.([]); });
  expect(screen.getByText("New account")).toBeInTheDocument();
});

it("does not infer continuity across resets omitted by archive sampling", async () => {
  vi.mocked(getProviderQuotaHistory).mockResolvedValue([{ label: "Week", samples:
    Array.from({ length: 64 }, (_, i) => ({ t: second - (63 - i) * 660, u: i % 2 ? .02 : .01, r: 2000 })),
  }]);
  const { container } = render(<ProviderQuotaHistory provider={provider} t={t} now={now} />);
  await screen.findByRole("img");
  expect(container.querySelectorAll("polyline, path")).toHaveLength(0);
  expect(container.querySelectorAll("line.quota-burndown-chart__observed")).toHaveLength(64);
});
