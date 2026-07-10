import { render, screen } from '@testing-library/react';
import { DetailLayout } from './index';

describe('DetailLayout', () => {
  it('renders rail and main regions with layout classes', () => {
    const { container } = render(<DetailLayout rail={<div>rail content</div>} main={<div>main content</div>} />);

    expect(screen.getByText('rail content')).toBeInTheDocument();
    expect(screen.getByText('main content')).toBeInTheDocument();
    expect(container.querySelector('.detail-layout')).toBeInTheDocument();
    expect(container.querySelector('.detail-rail')).toBeInTheDocument();
    expect(container.querySelector('.detail-main')).toBeInTheDocument();
  });
});
