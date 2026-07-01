import { createBrowserRouter } from 'react-router-dom';
import RootLayout from './pages/RootLayout';
import HomePage from './pages/HomePage';
import GeneralListPage from './pages/GeneralListPage';
import AcademicListPage from './pages/AcademicListPage';
import PaperListPage from './pages/PaperListPage';
import ItemDetailPage from './pages/ItemDetailPage';
import SearchAddPage from './pages/SearchAddPage';
import ItemFormPage from './pages/ItemFormPage';
import MyListsPage from './pages/MyListsPage';
import TagsCategoriesPage from './pages/TagsCategoriesPage';
import StaffPage from './pages/StaffPage';
import SettingsPage from './pages/SettingsPage';
import NotFoundPage from './pages/NotFoundPage';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'collections/general', element: <GeneralListPage /> },
      { path: 'collections/academic', element: <AcademicListPage /> },
      { path: 'collections/paper', element: <PaperListPage /> },
      { path: 'items/:id', element: <ItemDetailPage /> },
      { path: 'search/general', element: <SearchAddPage group="general" /> },
      { path: 'search/academic', element: <SearchAddPage group="academic" /> },
      { path: 'search/paper', element: <SearchAddPage group="paper" /> },
      { path: 'items/new/general', element: <ItemFormPage mode="create" group="general" /> },
      { path: 'items/new/academic', element: <ItemFormPage mode="create" group="academic" /> },
      { path: 'items/new/paper', element: <ItemFormPage mode="create" group="paper" /> },
      { path: 'items/:id/edit', element: <ItemFormPage mode="edit" /> },
      { path: 'mylists', element: <MyListsPage /> },
      { path: 'tags-categories', element: <TagsCategoriesPage /> },
      { path: 'staff', element: <StaffPage /> },
      { path: 'settings', element: <SettingsPage /> },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);
