import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { FiLink2, FiPaperclip } from 'react-icons/fi';
import { ResourceTabs, RatingStars, StatusSwitcher, TagList } from './index';

describe('shared components', () => {
  it('StatusSwitcher opens, closes on outside click, and reports selection', async () => {
    const onChange = vi.fn();
    render(
      <div>
        <StatusSwitcher value="not_started" onChange={onChange} />
        <button type="button">outside</button>
      </div>,
    );

    fireEvent.click(screen.getByRole('button', { name: /未着手/i }));
    expect(screen.getByRole('button', { name: /進行中/i })).toBeInTheDocument();

    fireEvent.mouseDown(screen.getByRole('button', { name: 'outside' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: /進行中/i })).toBeNull());

    fireEvent.click(screen.getByRole('button', { name: /未着手/i }));
    fireEvent.click(screen.getByRole('button', { name: /完了/i }));
    expect(onChange).toHaveBeenCalledWith('done');
  });

  it('RatingStars previews on hover and commits on click', () => {
    function Wrapper() {
      const [rating, setRating] = useState(2);
      return <RatingStars value={rating} onChange={setRating} />;
    }

    render(<Wrapper />);

    const stars = screen.getAllByRole('button');
    fireEvent.mouseOver(stars[3]);
    expect(stars[3].querySelector('.icon')).toHaveClass('is-full');

    fireEvent.click(stars[3]);
    expect(screen.getByText('4.0')).toBeInTheDocument();
  });

  it('TagList adds on Enter and cancels on Escape/blur', async () => {
    const Wrapper = () => {
      const [items, setItems] = useState([{ id: '1', label: 'alpha' }]);
      return (
        <TagList
          kind="tag"
          items={items}
          onAdd={(label) => setItems((current) => [...current, { id: label, label }])}
          onRemove={(id) => setItems((current) => current.filter((item) => item.id !== id))}
        />
      );
    };
    render(<Wrapper />);

    fireEvent.click(screen.getByRole('button', { name: /タグ追加/i }));
    const input = screen.getByPlaceholderText('タグ名を入力してEnter');
    fireEvent.change(input, { target: { value: 'beta' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.getByText('beta')).toBeInTheDocument();

    const cancelInput = screen.getByPlaceholderText('タグ名を入力してEnter');
    fireEvent.change(cancelInput, { target: { value: 'gamma' } });
    fireEvent.keyDown(cancelInput, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByDisplayValue('gamma')).not.toBeInTheDocument());
  });

  it('ResourceTabs switches visible content by state', () => {
    render(
      <ResourceTabs
        tabs={[
          { key: 'links', label: 'リンク', icon: FiLink2, content: <div>links panel</div> },
          { key: 'files', label: 'ファイル', icon: FiPaperclip, content: <div>files panel</div> },
        ]}
      />,
    );

    expect(screen.getByText('links panel')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab', { name: /ファイル/i }));
    expect(screen.getByText('files panel')).toBeInTheDocument();
  });
});
