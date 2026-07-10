import { NavLink } from "react-router-dom";
import { navigationSections, type NavigationItem, type NavigationSection } from "@/config/navigation";
import { cn } from "@/lib/cn";
import { ThemeToggle } from "./ThemeToggle";

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

  return (
    <NavLink
      to={item.to}
      className={({ isActive }) => cn("nav-item", item.indent && "indent", isActive && "active")}
    >
      <Icon className="icon" />
      <span>{item.label}</span>
      {typeof item.count === "number" ? <span className="count">{item.count}</span> : null}
    </NavLink>
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
      <ThemeToggle />
    </aside>
  );
}
