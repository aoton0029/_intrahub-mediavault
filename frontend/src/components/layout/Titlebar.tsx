import { Link, useMatches } from 'react-router-dom';
import type { AppRouteHandle } from '@/types/ui';

function hasHandle(value: unknown): value is AppRouteHandle {
  return Boolean(value) && typeof value === 'object' && 'title' in (value as Record<string, unknown>);
}

export function Breadcrumb() {
  const matches = useMatches();
  const currentHandle = matches.map((match) => match.handle).filter(hasHandle).at(-1);

  if (!currentHandle) {
    return null;
  }

  return (
    <div className="breadcrumb">
      {currentHandle.breadcrumb.map((item, index) => (
        <span key={`${item.label}-${index}`}>
          {index > 0 ? ' / ' : null}
          {item.to ? <Link to={item.to}>{item.label}</Link> : <span>{item.label}</span>}
        </span>
      ))}
    </div>
  );
}

export function Titlebar() {
  const matches = useMatches();
  const currentHandle = matches.map((match) => match.handle).filter(hasHandle).at(-1);

  if (!currentHandle) {
    return null;
  }

  return (
    <header className="titlebar">
      <div>
        <Breadcrumb />
        <h1>{currentHandle.title}</h1>
      </div>
      <div>{currentHandle.actions ?? null}</div>
    </header>
  );
}
