import { RouterProvider, createMemoryRouter } from 'react-router-dom';
import { render, screen } from '@testing-library/react';
import { AppShell } from './AppShell';

describe('AppShell', () => {
  it('renders breadcrumb and title from route handle metadata', () => {
    const router = createMemoryRouter(
      [
        {
          path: '/',
          element: <AppShell />,
          children: [
            {
              path: '/settings',
              element: <div>settings page</div>,
              handle: {
                title: '設定',
                breadcrumb: [{ label: '設定' }],
              },
            },
          ],
        },
      ],
      { initialEntries: ['/settings'] },
    );

    render(<RouterProvider router={router} />);

    expect(screen.getByRole('heading', { name: '設定' })).toBeInTheDocument();
    expect(screen.getByText('settings page')).toBeInTheDocument();
  });
});
