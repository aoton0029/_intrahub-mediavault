import { NavLink, useLocation } from 'react-router-dom';
import { navigationSections } from '@/config/navigation';
import { cn } from '@/lib/cn';
import { ThemeToggle } from './ThemeToggle';

function isActivePath(pathname: string, to: string) {
  return pathname === to || (to !== '/' && pathname.startsWith(`${to}/`));
}

export function Sidebar() {
  const location = useLocation();

  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="dot" />
        <span>MediaVault</span>
      </div>

      {navigationSections.map((section) => (
        <div className="nav-section" key={section.label}>
          <div className="nav-section-label">{section.label}</div>
          {section.items.map((item) => {
            const active = item.match
              ? item.match(location.pathname)
              : isActivePath(location.pathname, item.to);

            return (
              <NavLink
                key={`${section.label}-${item.label}`}
                to={item.to}
                className={cn('nav-item', active && 'active', item.indent && 'indent')}
                style={{ textDecoration: 'none' }}
              >
                <item.icon className="icon" />
                <span>{item.label}</span>
                {typeof item.count === 'number' ? <span className="count">{item.count}</span> : null}
              </NavLink>
            );
          })}
        </div>
      ))}

      <div className="mt-auto pt-4">
        <ThemeToggle />
      </div>
    </aside>
  );
}
