import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { QuickActionsSection } from "./QuickActionsSection";

describe("provider account actions", () => {
  it("keeps the dashboard while omitting an unsupported account switch", () => {
    render(<QuickActionsSection provider={{ dashboardUrl: "https://gemini.google.com/app", statusPageUrl: null, buyCreditsUrl: null }}
      busy={false} onOpenDashboard={vi.fn()} onOpenStatusPage={vi.fn()} onBuyCredits={vi.fn()} t={key=>key} />);
    expect(screen.getByRole("button", {name:"ActionUsageDashboard"})).toBeInTheDocument();
    expect(screen.queryByRole("button", {name:"ActionSwitchAccount"})).toBeNull();
  });
  it("retains existing providers' supported switch action", () => {
    render(<QuickActionsSection provider={{ dashboardUrl: "https://example.com", statusPageUrl: null, buyCreditsUrl: null }}
      busy={true} onSwitchAccount={vi.fn()} onOpenDashboard={vi.fn()} onOpenStatusPage={vi.fn()} onBuyCredits={vi.fn()} t={key=>key} />);
    expect(screen.getByRole("button", {name:"ActionSwitchAccount"})).toBeDisabled();
    expect(screen.getByRole("button", {name:"ActionUsageDashboard"})).toBeEnabled();
  });
});
