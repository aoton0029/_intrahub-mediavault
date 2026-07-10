import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { render, screen } from "@testing-library/react";
import { AppShell } from "./AppShell";

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

  it("renders titlebar actions", () => {
    renderWithRoute();
    expect(screen.getByRole("button", { name: "編集" })).toBeInTheDocument();
  });
});
