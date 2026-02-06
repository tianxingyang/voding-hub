export type RouteKey = 'global-config' | 'projects' | 'settings';

interface NavItem {
  key: RouteKey;
  label: string;
  icon: React.ReactNode;
}

export interface SidebarProps {
  currentRoute: RouteKey;
  onRouteChange: (route: RouteKey) => void;
}

const HomeIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
    <path strokeLinecap="round" strokeLinejoin="round" d="M3 10.75 12 3l9 7.75" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M5.5 9.75V20h13V9.75" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M10 20v-6h4v6" />
  </svg>
);

const FolderIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
    <path strokeLinecap="round" strokeLinejoin="round" d="M2.5 8.5A2.5 2.5 0 0 1 5 6h5l2 2h7A2.5 2.5 0 0 1 21.5 10.5v7A2.5 2.5 0 0 1 19 20H5a2.5 2.5 0 0 1-2.5-2.5z" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M2.5 10h19" />
  </svg>
);

const GearIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
    <circle cx="12" cy="12" r="3.25" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 2.75v2.1M12 19.15v2.1M4.85 4.85l1.5 1.5M17.65 17.65l1.5 1.5M2.75 12h2.1M19.15 12h2.1M4.85 19.15l1.5-1.5M17.65 6.35l1.5-1.5" />
  </svg>
);

const navItems: NavItem[] = [
  { key: 'global-config', label: 'Global Config', icon: <HomeIcon /> },
  { key: 'projects', label: 'Projects', icon: <FolderIcon /> },
  { key: 'settings', label: 'Settings', icon: <GearIcon /> },
];

export function Sidebar({ currentRoute, onRouteChange }: SidebarProps) {
  return (
    <aside className="sidebar">
      <p className="sidebar-title">Navigation</p>
      <nav className="sidebar-nav" aria-label="Main navigation">
        {navItems.map(({ key, label, icon }) => (
          <button
            key={key}
            type="button"
            className={`nav-item${key === currentRoute ? ' is-active' : ''}`}
            onClick={() => onRouteChange(key)}
            aria-current={key === currentRoute ? 'page' : undefined}
          >
            <span className="nav-icon" aria-hidden="true">{icon}</span>
            <span>{label}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
