import { useEffect } from 'react';
import { useConfigStore, type Project, type ToolType } from '../stores/configStore';

const TOOL_LABELS: Record<ToolType, string> = {
  ClaudeCode: 'Claude',
  Codex: 'Codex',
  Gemini: 'Gemini',
  OpenCode: 'OpenCode',
};

interface Props {
  selectedId: number | null;
  onSelect: (project: Project) => void;
  onAdd: () => void;
}

export function ProjectList({ selectedId, onSelect, onAdd }: Props) {
  const { projects, loading, fetchProjects } = useConfigStore();

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  return (
    <div className="project-list">
      <div className="project-list-header">
        <span className="project-list-title">Projects</span>
        <button type="button" className="mcp-add-btn" onClick={onAdd}>
          + Add
        </button>
      </div>
      {loading.projects > 0 && projects.length === 0 && (
        <p className="mcp-loading">Loading projects...</p>
      )}
      {projects.length === 0 && loading.projects === 0 && (
        <p className="mcp-empty">No projects added yet.</p>
      )}
      {projects.map((p) => (
        <button
          key={p.id}
          type="button"
          className={`project-row${p.id === selectedId ? ' is-selected' : ''}`}
          onClick={() => onSelect(p)}
        >
          <div className="project-row-info">
            <span className="project-row-name">{p.name}</span>
            <span className="project-row-path" title={p.path}>{p.path}</span>
          </div>
          <div className="project-tool-badges">
            {p.tools.map((t) => (
              <span key={t} className="tool-badge">
                {TOOL_LABELS[t]}
              </span>
            ))}
          </div>
        </button>
      ))}
    </div>
  );
}
