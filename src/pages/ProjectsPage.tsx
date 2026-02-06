import { useState } from 'react';
import { ProjectList } from '../components/ProjectList';
import { AddProjectDialog } from '../components/AddProjectDialog';
import { ProjectConfigOverview } from '../components/ProjectConfigOverview';
import { RemoveProjectDialog } from '../components/RemoveProjectDialog';
import { useConfigStore, type Project } from '../stores/configStore';

export function ProjectsPage() {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<Project | null>(null);
  const { fetchProjects, projects } = useConfigStore();
  const selected = selectedId != null ? projects.find((p) => p.id === selectedId) ?? null : null;

  const handleRemoved = () => {
    setSelectedId(null);
    setRemoveTarget(null);
  };

  return (
    <section className="content-panel">
      <h1 className="content-title">Projects</h1>
      <p className="content-description">
        Browse and manage project workspaces with tool-specific configurations.
      </p>

      <div className="mcp-layout" style={{ marginTop: 20 }}>
        <div className="mcp-layout-list">
          <ProjectList
            selectedId={selectedId}
            onSelect={(p) => setSelectedId(p.id)}
            onAdd={() => setShowAdd(true)}
          />
        </div>
        {selected && (
          <div className="mcp-layout-detail">
            <ProjectConfigOverview
              key={selected.id}
              project={selected}
              onRemove={() => setRemoveTarget({ ...selected })}
            />
          </div>
        )}
      </div>

      {showAdd && (
        <AddProjectDialog
          onClose={() => setShowAdd(false)}
          onAdded={() => fetchProjects()}
        />
      )}

      {removeTarget && (
        <RemoveProjectDialog
          project={removeTarget}
          onClose={() => setRemoveTarget(null)}
          onRemoved={handleRemoved}
        />
      )}
    </section>
  );
}
