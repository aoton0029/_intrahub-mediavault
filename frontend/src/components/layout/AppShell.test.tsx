import { useMemo } from "react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { render, screen } from "@testing-library/react";
import { AppShell } from "./AppShell";
import { usePageChrome } from "./usePageChrome";

function DynamicChromePage() {
  const pageChrome = useMemo(() => ({
    breadcrumbs: [
      { label: "一般メディア", to: "/media" },
      { label: "映画" },
    ],
    actions: <button>編集する</button>,
  }), []);
  usePageChrome(pageChrome);

  return <div>Dynamic Body</div>;
}

function renderWithRoute(path = "/media") {
  const router = createMemoryRouter(
    [
      {
        path: "/",
        element: <AppShell title="テスト" actions={<button>編集</button>} />,
        children: [
          { path: "media", element: <div>Outlet Body</div> },
          { path: "settings", element: <div>Settings</div> },
        ],
      },
    ],
    { initialEntries: [path] },
  );

  return render(<RouterProvider router={router} />);
}

describe("AppShell", () => {
  it("renders sidebar, titlebar and outlet", () => {
    renderWithRoute();
    expect(screen.getByText("MediaVault")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "テスト" })).toBeInTheDocument();
    expect(screen.getByText("Outlet Body")).toBeInTheDocument();
  });

  it("marks active nav item", () => {
    renderWithRoute("/settings");
    expect(screen.getByRole("link", { name: /設定/ })).toHaveAttribute("aria-current", "page");
  });

  it("marks only the all-media item active on /media", () => {
    renderWithRoute("/media");

    expect(screen.getByRole("link", { name: /すべて/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: /映画/ })).not.toHaveAttribute("aria-current");
    expect(screen.getByRole("link", { name: /アニメ/ })).not.toHaveAttribute("aria-current");
  });

  it("marks only the matching media_type item active on /media?media_type=movie", () => {
    renderWithRoute("/media?media_type=movie");

    expect(screen.getByRole("link", { name: /映画/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: /すべて/ })).not.toHaveAttribute("aria-current");
    expect(screen.getByRole("link", { name: /アニメ/ })).not.toHaveAttribute("aria-current");
  });

  it("marks only the matching media_type item active on /media?media_type=anime", () => {
    renderWithRoute("/media?media_type=anime");

    expect(screen.getByRole("link", { name: /アニメ/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: /映画/ })).not.toHaveAttribute("aria-current");
    expect(screen.getByRole("link", { name: /すべて/ })).not.toHaveAttribute("aria-current");
  });

  it("renders titlebar actions", () => {
    renderWithRoute();
    expect(screen.getByRole("button", { name: "編集" })).toBeInTheDocument();
  });

  it("prefers page-provided chrome over route handles", () => {
    const router = createMemoryRouter(
      [
        {
          path: "/",
          element: <AppShell />,
          children: [
            {
              path: "media/:id",
              element: <DynamicChromePage />,
              handle: {
                breadcrumbs: [{ label: "一般メディア", to: "/media" }, { label: "アニメ" }],
                actions: <button>静的アクション</button>,
              },
            },
          ],
        },
      ],
      { initialEntries: ["/media/1"] },
    );

    render(<RouterProvider router={router} />);

    expect(document.querySelector(".breadcrumb")?.textContent).toContain("一般メディア / 映画");
    expect(screen.getByRole("button", { name: "編集する" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "静的アクション" })).not.toBeInTheDocument();
  });
});
