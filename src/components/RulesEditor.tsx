import { useState, useEffect, useRef, useCallback } from 'react';
import { useConfigStore, TOOLS, type ToolType } from '../stores/configStore';

const TOOL_PATHS: Record<ToolType, string> = {
  ClaudeCode: '~/.claude/CLAUDE.md',
  Codex: '~/.codex/AGENTS.md',
  Gemini: '~/.gemini/GEMINI.md',
  OpenCode: '~/.config/opencode/AGENTS.md',
};

interface ConflictState {
  tool: ToolType;
  incoming: string;
}

type Drafts = Partial<Record<ToolType, string>>;

export function RulesEditor() {
  const { rules, fetchRules, saveRules, loading } = useConfigStore();

  const [activeTool, setActiveTool] = useState<ToolType>('ClaudeCode');
  const [drafts, setDrafts] = useState<Drafts>({});
  const [conflict, setConflict] = useState<ConflictState | null>(null);
  const baseRef = useRef<Partial<Record<ToolType, string>>>({});

  useEffect(() => { fetchRules(activeTool); }, [activeTool, fetchRules]);

  const serverContent = rules[activeTool] ?? '';
  const draft = drafts[activeTool];
  const isDirty = draft !== undefined;

  useEffect(() => {
    if (!isDirty) {
      baseRef.current[activeTool] = serverContent;
      return;
    }
    if (serverContent === draft) {
      baseRef.current[activeTool] = serverContent;
      setDrafts((d) => { const n = { ...d }; delete n[activeTool]; return n; });
      return;
    }
    const base = baseRef.current[activeTool] ?? '';
    if (serverContent !== base) {
      setConflict({ tool: activeTool, incoming: serverContent });
    }
  }, [serverContent, activeTool, isDirty, draft]);

  const clearDraft = useCallback((tool: ToolType) => {
    setDrafts((d) => { const n = { ...d }; delete n[tool]; return n; });
  }, []);

  const handleSave = useCallback(async () => {
    const d = drafts[activeTool];
    if (d === undefined) return;
    const tool = activeTool;
    const content = d;
    await saveRules(tool, content);
    baseRef.current[tool] = content;
    setDrafts((prev) => {
      if (prev[tool] === content) {
        const n = { ...prev }; delete n[tool]; return n;
      }
      return prev;
    });
  }, [activeTool, drafts, saveRules]);

  const handleDiscard = useCallback(() => {
    clearDraft(activeTool);
    setConflict(null);
    baseRef.current[activeTool] = rules[activeTool] ?? '';
  }, [activeTool, rules, clearDraft]);

  const resolveConflict = useCallback((keepMine: boolean) => {
    if (!conflict) return;
    const t = conflict.tool;
    if (keepMine) {
      baseRef.current[t] = conflict.incoming;
    } else {
      clearDraft(t);
      baseRef.current[t] = rules[t] ?? '';
    }
    setConflict(null);
  }, [conflict, rules, clearDraft]);

  const displayContent = draft ?? serverContent;
  const isLoading = loading.rules > 0;
  const pathHint = TOOL_PATHS[activeTool];

  return (
    <div className="rules-editor">
      <div className="rules-tool-tabs">
        {TOOLS.map((tool) => (
          <button
            key={tool}
            type="button"
            className={`rules-tool-tab${activeTool === tool ? ' is-active' : ''}${drafts[tool] !== undefined ? ' has-draft' : ''}`}
            onClick={() => setActiveTool(tool)}
          >
            {tool}
          </button>
        ))}
      </div>

      <div className="rules-toolbar">
        <code className="rules-path">{pathHint}</code>
        {isDirty && (
          <div className="rules-actions">
            <button type="button" className="btn-secondary btn-sm" onClick={handleDiscard}>Discard</button>
            <button type="button" className="btn-primary btn-sm" onClick={handleSave}>Save</button>
          </div>
        )}
      </div>

      <textarea
        className="rules-textarea"
        value={displayContent}
        onChange={(e) => setDrafts((d) => ({ ...d, [activeTool]: e.target.value }))}
        disabled={isLoading}
        spellCheck={false}
        placeholder={isLoading ? 'Loading...' : 'No rules file found. Start typing to create one.'}
      />

      {conflict && (
        <div className="modal-overlay">
          <div className="modal-box">
            <h3 className="modal-title">External Changes Detected</h3>
            <p className="modal-desc">
              The rules file for {conflict.tool} was modified externally. Choose how to proceed.
            </p>
            <div className="form-actions">
              <div className="form-actions-left" />
              <div className="form-actions-right">
                <button type="button" className="btn-secondary" onClick={() => resolveConflict(false)}>
                  Load external
                </button>
                <button type="button" className="btn-primary" onClick={() => resolveConflict(true)}>
                  Keep mine
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
