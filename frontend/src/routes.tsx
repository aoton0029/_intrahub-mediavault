import { createBrowserRouter, Link } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";
import { AcademicBookListPage } from "@/pages/AcademicBookListPage";
import { MediaDetailPage } from "@/pages/MediaDetailPage";
import { MediaListPage } from "@/pages/MediaListPage";
import { MediaSearchPage } from "@/pages/MediaSearchPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { CollageToolPage } from "@/pages/CollageToolPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage />, handle: { title: "ホーム" } },
      {
        path: "media",
        element: <MediaListPage />,
        handle: {
          title: "一般メディア",
          actions: <Link className="btn btn-accent" to="/media/search">＋ 作品を追加</Link>,
        },
      },
      {
        path: "media/:id",
        element: <MediaDetailPage />,
      },
      {
        path: "media/search",
        element: <MediaSearchPage />,
        handle: {
          title: "検索して追加",
          actions: <Link className="btn btn-accent btn-sm" to="/media/new">手動で入力する</Link>,
        },
      },
      {
        path: "academic-books",
        element: <AcademicBookListPage />,
        handle: {
          title: "学術書・専門書",
          actions: <Link className="btn btn-accent" to="/academic-books/search">＋ 作品を追加</Link>,
        },
      },
      { path: "tools/collage", element: <CollageToolPage />, handle: { title: "並べてシェア" } },
      { path: "settings", element: <SettingsPage />, handle: { title: "設定" } },
    ],
  },
]);
