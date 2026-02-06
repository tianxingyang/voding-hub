import { useState } from 'react';
import { Layout } from './components/Layout';
import type { RouteKey } from './components/Sidebar';
import { GlobalConfigPage } from './pages/GlobalConfigPage';
import { ProjectsPage } from './pages/ProjectsPage';
import { SettingsPage } from './pages/SettingsPage';
import './App.css';

const routeComponents: Record<RouteKey, React.FC> = {
  'global-config': GlobalConfigPage,
  projects: ProjectsPage,
  settings: SettingsPage,
};

function App() {
  const [currentRoute, setCurrentRoute] = useState<RouteKey>('global-config');
  const PageComponent = routeComponents[currentRoute];

  return (
    <Layout currentRoute={currentRoute} onRouteChange={setCurrentRoute}>
      <PageComponent />
    </Layout>
  );
}

export default App;
