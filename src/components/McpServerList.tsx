import { useEffect, useState } from 'react';
import { useConfigStore, TOOLS, type ToolType, type McpServer } from '../stores/configStore';

interface McpServerListProps {
  selectedTool?: ToolType;
  selectedName?: string;
  onSelect: (tool: ToolType, server: McpServer) => void;
  onAdd: (tool: ToolType) => void;
}

export function McpServerList({ selectedTool, selectedName, onSelect, onAdd }: McpServerListProps) {
  const { mcpServers, fetchAllMcpServers, loading } = useConfigStore();
  const [collapsed, setCollapsed] = useState<Partial<Record<ToolType, boolean>>>({});

  useEffect(() => { fetchAllMcpServers(); }, [fetchAllMcpServers]);

  const toggle = (tool: ToolType) =>
    setCollapsed((p) => ({ ...p, [tool]: !p[tool] }));

  return (
    <div className="mcp-list">
      {loading.mcp > 0 && <p className="mcp-loading">Loading...</p>}
      {TOOLS.map((tool) => {
        const servers = mcpServers[tool];
        const isCollapsed = !!collapsed[tool];
        return (
          <section key={tool} className="mcp-tool-section">
            <div className="mcp-tool-header">
              <button type="button" className="mcp-tool-toggle" onClick={() => toggle(tool)}>
                <span className={`mcp-chevron${isCollapsed ? ' is-collapsed' : ''}`}>&#9660;</span>
                <span className="mcp-tool-name">{tool}</span>
                <span className="mcp-tool-count">{servers.length}</span>
              </button>
              <button type="button" className="mcp-add-btn"
                onClick={() => onAdd(tool)}>+ Add</button>
            </div>
            {!isCollapsed && (
              <div className="mcp-tool-body">
                {servers.length === 0 ? (
                  <p className="mcp-empty">No MCP servers configured</p>
                ) : servers.map((s) => (
                  <div
                    key={s.name}
                    role="button"
                    tabIndex={0}
                    className={`mcp-row${selectedTool === tool && selectedName === s.name ? ' is-selected' : ''}`}
                    onClick={() => onSelect(tool, s)}
                    onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(tool, s); } }}
                  >
                    <div className="mcp-row-info">
                      <span className="mcp-row-name">{s.name}</span>
                      <span className="mcp-row-cmd" title={s.command}>{s.command}</span>
                    </div>
                    {tool === 'Codex' && (
                      <span className={`mcp-badge${s.enabled ? ' is-on' : ''}`}>
                        {s.enabled ? 'ON' : 'OFF'}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </section>
        );
      })}
    </div>
  );
}
