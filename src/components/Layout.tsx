import type { ReactNode } from 'react';
import { Sidebar, type RouteKey } from './Sidebar';

export interface LayoutProps {
  children: ReactNode;
  currentRoute: RouteKey;
  onRouteChange: (route: RouteKey) => void;
}

export function Layout({ children, currentRoute, onRouteChange }: LayoutProps) {
  return (
    <div className="app-shell">
      <Sidebar currentRoute={currentRoute} onRouteChange={onRouteChange} />
      <main className="main-content">{children}</main>
    </div>
  );
}
