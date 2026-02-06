import { useState } from 'react';
import { McpServerList } from '../components/McpServerList';
import { McpServerForm } from '../components/McpServerForm';
import { CopyMcpDialog } from '../components/CopyMcpDialog';
import { SkillList } from '../components/SkillList';
import { SkillDetail } from '../components/SkillDetail';
import { CopySkillDialog } from '../components/CopySkillDialog';
import { RulesEditor } from '../components/RulesEditor';
import { useConfigStore, type ToolType, type McpServer, type Skill } from '../stores/configStore';

type Tab = 'mcp' | 'skills' | 'rules';

type McpView =
  | { kind: 'list' }
  | { kind: 'edit'; tool: ToolType; server: McpServer }
  | { kind: 'new'; tool: ToolType };

type SkillView =
  | { kind: 'list' }
  | { kind: 'detail'; tool: ToolType; skill: Skill };

export function GlobalConfigPage() {
  const [tab, setTab] = useState<Tab>('mcp');
  const [mcpView, setMcpView] = useState<McpView>({ kind: 'list' });
  const [skillView, setSkillView] = useState<SkillView>({ kind: 'list' });
  const [copyMcp, setCopyMcp] = useState<{ tool: ToolType; name: string } | null>(null);
  const [copySkill, setCopySkill] = useState<{ tool: ToolType; name: string } | null>(null);
  const { saveMcpServer, deleteMcpServer, deleteSkill } = useConfigStore();

  const handleMcpSave = async (tool: ToolType, server: McpServer) => {
    await saveMcpServer(tool, server);
    setMcpView({ kind: 'list' });
  };

  const handleMcpDelete = async (tool: ToolType, name: string) => {
    await deleteMcpServer(tool, name);
    setMcpView({ kind: 'list' });
  };

  const handleSkillDelete = async (tool: ToolType, name: string) => {
    await deleteSkill(tool, name);
    setSkillView({ kind: 'list' });
  };

  const switchTab = (t: Tab) => {
    setTab(t);
    setMcpView({ kind: 'list' });
    setSkillView({ kind: 'list' });
  };

  return (
    <section className="content-panel">
      <h1 className="content-title">Global Config</h1>
      <p className="content-description">
        Manage MCP servers and Skills across all AI coding tools.
      </p>

      <div className="config-tabs">
        <button type="button" className={`config-tab${tab === 'mcp' ? ' is-active' : ''}`}
          onClick={() => switchTab('mcp')}>MCP Servers</button>
        <button type="button" className={`config-tab${tab === 'skills' ? ' is-active' : ''}`}
          onClick={() => switchTab('skills')}>Skills</button>
        <button type="button" className={`config-tab${tab === 'rules' ? ' is-active' : ''}`}
          onClick={() => switchTab('rules')}>Rules</button>
      </div>

      {tab === 'mcp' && (
        <div className="mcp-layout">
          <div className="mcp-layout-list">
            <McpServerList
              selectedTool={mcpView.kind === 'edit' ? mcpView.tool : mcpView.kind === 'new' ? mcpView.tool : undefined}
              selectedName={mcpView.kind === 'edit' ? mcpView.server.name : undefined}
              onSelect={(tool, server) => setMcpView({ kind: 'edit', tool, server })}
              onAdd={(tool) => setMcpView({ kind: 'new', tool })}
            />
          </div>
          {mcpView.kind !== 'list' && (
            <div className="mcp-layout-detail">
              <McpServerForm
                key={mcpView.kind === 'edit' ? `${mcpView.tool}-${mcpView.server.name}` : `new-${mcpView.tool}`}
                tool={mcpView.tool}
                server={mcpView.kind === 'edit' ? mcpView.server : undefined}
                isNew={mcpView.kind === 'new'}
                onSave={(s) => handleMcpSave(mcpView.tool, s)}
                onDelete={mcpView.kind === 'edit' ? () => handleMcpDelete(mcpView.tool, mcpView.server.name) : undefined}
                onCancel={() => setMcpView({ kind: 'list' })}
                onCopy={mcpView.kind === 'edit' ? () => setCopyMcp({ tool: mcpView.tool, name: mcpView.server.name }) : undefined}
              />
            </div>
          )}
        </div>
      )}

      {tab === 'skills' && (
        <div className="mcp-layout">
          <div className="mcp-layout-list">
            <SkillList
              selectedTool={skillView.kind === 'detail' ? skillView.tool : undefined}
              selectedName={skillView.kind === 'detail' ? skillView.skill.name : undefined}
              onSelect={(tool, skill) => setSkillView({ kind: 'detail', tool, skill })}
            />
          </div>
          {skillView.kind === 'detail' && (
            <div className="mcp-layout-detail">
              <SkillDetail
                key={`${skillView.tool}-${skillView.skill.name}`}
                skill={skillView.skill}
                tool={skillView.tool}
                onDelete={() => handleSkillDelete(skillView.tool, skillView.skill.name)}
                onCopy={() => setCopySkill({ tool: skillView.tool, name: skillView.skill.name })}
                onClose={() => setSkillView({ kind: 'list' })}
              />
            </div>
          )}
        </div>
      )}

      {tab === 'rules' && <RulesEditor />}

      {copyMcp && (
        <CopyMcpDialog
          serverName={copyMcp.name}
          sourceTool={copyMcp.tool}
          onClose={() => setCopyMcp(null)}
        />
      )}

      {copySkill && (
        <CopySkillDialog
          skillName={copySkill.name}
          sourceTool={copySkill.tool}
          onClose={() => setCopySkill(null)}
        />
      )}
    </section>
  );
}
