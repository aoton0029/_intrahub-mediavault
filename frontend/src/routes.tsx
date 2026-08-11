import { createBrowserRouter } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { HomePage } from "@/pages/HomePage";
import { AcademicBookListPage } from "@/pages/AcademicBookListPage";
import { AcademicBookDetailPage } from "@/pages/AcademicBookDetailPage";
import { AcademicBookSearchPage } from "@/pages/AcademicBookSearchPage";
import { MediaDetailPage } from "@/pages/MediaDetailPage";
import { MediaListPage } from "@/pages/MediaListPage";
import { MediaSearchPage } from "@/pages/MediaSearchPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { CollageToolPage } from "@/pages/CollageToolPage";
import { YearlyMediaPage } from "@/pages/YearlyMediaPage";
import { MyListPage } from "@/pages/MyListPage";

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
          title: "メディア",
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
        },
      },
      {
        path: "academic-books",
        element: <AcademicBookListPage />,
        handle: {
          title: "学術書・専門書",
        },
      },
      {
        path: "academic-books/:id",
        element: <AcademicBookDetailPage />,
      },
      {
        path: "academic-books/search",
        element: <AcademicBookSearchPage />,
        handle: {
          title: "検索して追加",
        },
      },
      {
        path: "mylists",
        element: <MyListPage />,
        handle: { title: "マイリスト" },
      },
      { path: "collection/yearly", element: <YearlyMediaPage />, handle: { title: "年別" } },
      { path: "tools/collage", element: <CollageToolPage />, handle: { title: "並べてシェア" } },
      { path: "settings", element: <SettingsPage />, handle: { title: "設定" } },
    ],
  },
]);
