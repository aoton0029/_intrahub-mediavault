import { cn } from "./cn";

describe("cn", () => {
  it("merges tailwind classes", () => {
    expect(cn("px-2", "px-4", "text-sm")).toBe("px-4 text-sm");
  });

  it("handles conditional values", () => {
    const shouldHide = false;
    expect(cn("base", shouldHide ? "hidden" : undefined, { active: true, idle: false })).toBe("base active");
  });
});
