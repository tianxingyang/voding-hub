import { useState } from 'react';
import { TOOLS, type ToolType, useConfigStore } from '../stores/configStore';

interface CopyMcpDialogProps {
  serverName: string;
  sourceTool: ToolType;
  onClose: () => void;
}

type CopyStatus = 'idle' | 'pending' | 'success' | 'skipped' | 'error';

interface ToolResult {
  status: CopyStatus;
  warnings?: string[];
}

export function CopyMcpDialog({ serverName, sourceTool, onClose }: CopyMcpDialogProps) {
  const copyMcpServer = useConfigStore((s) => s.copyMcpServer);
  const targets = TOOLS.filter((t) => t !== sourceTool);

  const [selected, setSelected] = useState<Set<ToolType>>(new Set());
  const [results, setResults] = useState<Partial<Record<ToolType, ToolResult>>>({});
  const [copying, setCopying] = useState(false);

  const toggle = (tool: ToolType) => {
    const next = new Set(selected);
    next.has(tool) ? next.delete(tool) : next.add(tool);
    setSelected(next);
  };

  const handleCopy = async () => {
    if (!selected.size) return;
    setCopying(true);
    for (const t of selected) {
      setResults((p) => ({ ...p, [t]: { status: 'pending' } }));
    }
    await Promise.all(
      [...selected].map(async (tool) => {
        try {
          const res = await copyMcpServer(sourceTool, tool, serverName);
          setResults((p) => ({
            ...p,
            [tool]: {
              status: res.skipped ? 'skipped' : 'success',
              warnings: res.warnings.length ? res.warnings : undefined,
            },
          }));
        } catch (e) {
          setResults((p) => ({ ...p, [tool]: { status: 'error', warnings: [String(e)] } }));
        }
      }),
    );
    setCopying(false);
  };

  const done = selected.size > 0 && [...selected].every(
    (t) => results[t] && results[t]!.status !== 'idle' && results[t]!.status !== 'pending',
  );

  return (
    <div className="modal-overlay" onClick={copying ? undefined : onClose}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">Copy "{serverName}"</h2>
        <p className="modal-desc">Select target tools:</p>

        <div className="copy-targets">
          {targets.map((tool) => {
            const r = results[tool];
            const isDone = r?.status === 'success' || r?.status === 'skipped';
            return (
              <div key={tool} className={`copy-target${selected.has(tool) ? ' is-checked' : ''}`}>
                <label className="copy-target-label">
                  <input type="checkbox" checked={selected.has(tool)}
                    onChange={() => toggle(tool)} disabled={copying || isDone} />
                  {tool}
                </label>
                {r && r.status !== 'idle' && (
                  <span className={`copy-status copy-status--${r.status}`}>{r.status}</span>
                )}
              </div>
            );
          })}
        </div>

        {/* Warnings display (task 14.4) */}
        {Object.entries(results).some(([, r]) => r?.warnings?.length) && (
          <div className="copy-warnings">
            {Object.entries(results).map(([tool, r]) =>
              r?.warnings?.map((w, i) => (
                <p key={`${tool}-${i}`} className="copy-warning-item">{tool}: {w}</p>
              )),
            )}
          </div>
        )}

        <div className="form-actions">
          <div className="form-actions-left" />
          <div className="form-actions-right">
            <button type="button" className="btn-secondary" onClick={onClose} disabled={copying}>
              {done ? 'Close' : 'Cancel'}
            </button>
            {!done && (
              <button type="button" className="btn-primary" onClick={handleCopy}
                disabled={copying || !selected.size}>
                {copying ? 'Copying...' : 'Copy'}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}