import { useEffect, useState } from 'react';
import { useConfigStore, TOOLS, type ToolType, type Skill } from '../stores/configStore';
import { validateSkill } from './skillValidation';

interface SkillListProps {
  selectedTool?: ToolType;
  selectedName?: string;
  onSelect: (tool: ToolType, skill: Skill) => void;
}

export function SkillList({ selectedTool, selectedName, onSelect }: SkillListProps) {
  const { skills, fetchAllSkills, loading } = useConfigStore();
  const [collapsed, setCollapsed] = useState<Partial<Record<ToolType, boolean>>>({});

  useEffect(() => { fetchAllSkills(); }, [fetchAllSkills]);

  const toggle = (tool: ToolType) =>
    setCollapsed((p) => ({ ...p, [tool]: !p[tool] }));

  return (
    <div className="mcp-list">
      {loading.skills > 0 && <p className="mcp-loading">Loading...</p>}
      {TOOLS.map((tool) => {
        const items = skills[tool];
        const isCollapsed = !!collapsed[tool];
        return (
          <section key={tool} className="mcp-tool-section">
            <div className="mcp-tool-header">
              <button type="button" className="mcp-tool-toggle" onClick={() => toggle(tool)}>
                <span className={`mcp-chevron${isCollapsed ? ' is-collapsed' : ''}`}>&#9660;</span>
                <span className="mcp-tool-name">{tool}</span>
                <span className="mcp-tool-count">{items.length}</span>
              </button>
            </div>
            {!isCollapsed && (
              <div className="mcp-tool-body">
                {items.length === 0 ? (
                  <p className="mcp-empty">No skills found</p>
                ) : items.map((s) => {
                  const v = validateSkill(s);
                  return (
                    <div
                      key={s.name}
                      role="button"
                      tabIndex={0}
                      className={`mcp-row${selectedTool === tool && selectedName === s.name ? ' is-selected' : ''}`}
                      onClick={() => onSelect(tool, s)}
                      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(tool, s); } }}
                    >
                      <div className="mcp-row-info">
                        <span className="mcp-row-name">
                          <span
                            className={`skill-dot${v.valid ? ' is-valid' : ' is-invalid'}`}
                            title={v.valid ? 'Valid' : v.errors.join('; ')}
                          />
                          {s.name}
                        </span>
                        {s.description && (
                          <span className="mcp-row-cmd" title={s.description}>{s.description}</span>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </section>
        );
      })}
    </div>
  );
}
