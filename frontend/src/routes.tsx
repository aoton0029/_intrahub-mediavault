import { createBrowserRouter } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage />, handle: { title: "ホーム" } },
      { path: "media", element: <div>一般メディア一覧のプレースホルダ</div>, handle: { title: "一般メディア" } },
      { path: "settings", element: <div>設定画面のプレースホルダ</div>, handle: { title: "設定" } },
    ],
  },
]);
