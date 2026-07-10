import { createBrowserRouter, Link } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";
import { AcademicBookListPage } from "@/pages/AcademicBookListPage";
import { MediaListPage } from "@/pages/MediaListPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage />, handle: { title: "ホーム" } },
      { path: "media", element: <MediaListPage />, handle: { title: "一般メディア" } },
      {
        path: "academic-books",
        element: <AcademicBookListPage />,
        handle: {
          title: "学術書・専門書",
          actions: <Link className="btn btn-accent" to="/academic-books/search">＋ 作品を追加</Link>,
        },
      },
      { path: "settings", element: <div>設定画面のプレースホルダ</div>, handle: { title: "設定" } },
    ],
  },
]);
