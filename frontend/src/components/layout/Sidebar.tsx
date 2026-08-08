import { Link, useLocation } from "react-router-dom";
import { navigationSections, type NavigationItem, type NavigationSection } from "@/config/navigation";
import { cn } from "@/lib/cn";

function isNavItemActive(currentPathname: string, currentSearch: string, target: string) {
  const [targetPathname, targetSearch = ""] = target.split("?");
  const currentParams = new URLSearchParams(currentSearch);
  const targetParams = new URLSearchParams(targetSearch);

  if (currentPathname !== targetPathname) {
    return false;
  }

  if ([...targetParams.keys()].length === 0) {
    return [...currentParams.keys()].length === 0;
  }

  if ([...currentParams.keys()].length !== [...targetParams.keys()].length) {
    return false;
  }

  for (const [key, value] of targetParams.entries()) {
    if (currentParams.get(key) !== value) {
      return false;
    }
  }

  return true;
}

function Brand() {
  return (
    <div className="brand">
      <span className="dot" />
      <span>MediaVault</span>
    </div>
  );
}

function NavItem({ item }: { item: NavigationItem }) {
  const Icon = item.icon;
  const location = useLocation();
  const isActive = isNavItemActive(location.pathname, location.search, item.to);

  return (
    <Link
      to={item.to}
      aria-current={isActive ? "page" : undefined}
      className={cn("nav-item", item.indent && "indent", isActive && "active")}
    >
      <Icon className="icon" />
      <span>{item.label}</span>
    </Link>
  );
}

function NavSection({ section }: { section: NavigationSection }) {
  return (
    <div className="nav-section" style={section.grow ? { marginTop: "auto" } : undefined}>
      {section.label ? <div className="nav-section-label">{section.label}</div> : null}
      {section.items.map((item) => (
        <NavItem key={item.to} item={item} />
      ))}
    </div>
  );
}

export function Sidebar() {
  return (
    <aside className="sidebar">
      <Brand />
      {navigationSections.map((section, index) => (
        <NavSection key={section.label ?? `section-${index}`} section={section} />
      ))}
    </aside>
  );
}
