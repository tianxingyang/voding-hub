import { useEffect, useState } from 'react';
import {
  useConfigStore,
  type Project,
  type ProjectConfigSummary,
  type ToolType,
} from '../stores/configStore';

const TOOL_LABELS: Record<ToolType, string> = {
  ClaudeCode: 'Claude Code',
  Codex: 'Codex',
  Gemini: 'Gemini',
  OpenCode: 'OpenCode',
};

interface Props {
  project: Project;
  onRemove: () => void;
}

export function ProjectConfigOverview({ project, onRemove }: Props) {
  const { getProjectConfigSummary } = useConfigStore();
  const [summaries, setSummaries] = useState<ProjectConfigSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    getProjectConfigSummary(project.path)
      .then((s) => { if (!cancelled) setSummaries(s); })
      .catch((e) => { if (!cancelled) { setSummaries([]); setError(String(e)); } })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [project.path, getProjectConfigSummary]);

  return (
    <div className="project-overview">
      <div className="project-overview-header">
        <h2 className="project-overview-name">{project.name}</h2>
        <button type="button" className="btn-danger btn-sm" onClick={onRemove}>
          Remove
        </button>
      </div>
      <p className="project-overview-path" title={project.path}>{project.path}</p>

      {loading ? (
        <p className="mcp-loading">Loading config summary...</p>
      ) : error ? (
        <p className="error-msg">{error}</p>
      ) : summaries.length === 0 ? (
        <p className="mcp-empty">No tool configurations detected.</p>
      ) : (
        <div className="project-summary-grid">
          {summaries.map((s) => (
            <div key={s.tool} className="project-summary-card">
              <span className="project-summary-tool">
                {TOOL_LABELS[s.tool]}
              </span>
              <div className="project-summary-stats">
                <span className="project-stat">
                  <strong>{s.mcp_count}</strong> MCP
                </span>
                <span className="project-stat">
                  <strong>{s.skills_count}</strong> Skills
                </span>
                <span className={`project-stat${s.has_rules ? '' : ' is-dim'}`}>
                  Rules {s.has_rules ? '✓' : '—'}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
