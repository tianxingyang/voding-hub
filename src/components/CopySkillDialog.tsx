import { useState } from 'react';
import { TOOLS, type ToolType, useConfigStore } from '../stores/configStore';

interface CopySkillDialogProps {
  skillName: string;
  sourceTool: ToolType;
  onClose: () => void;
}

type CopyStatus = 'idle' | 'pending' | 'success' | 'skipped' | 'error';

export function CopySkillDialog({ skillName, sourceTool, onClose }: CopySkillDialogProps) {
  const copySkill = useConfigStore((s) => s.copySkill);
  const targets = TOOLS.filter((t) => t !== sourceTool);

  const [selected, setSelected] = useState<Set<ToolType>>(new Set());
  const [results, setResults] = useState<Partial<Record<ToolType, CopyStatus>>>({});
  const [errors, setErrors] = useState<Partial<Record<ToolType, string>>>({});
  const [copying, setCopying] = useState(false);

  const toggle = (tool: ToolType) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(tool) ? next.delete(tool) : next.add(tool);
      return next;
    });
  };

  const handleCopy = async () => {
    if (!selected.size) return;
    setCopying(true);
    for (const t of selected) setResults((p) => ({ ...p, [t]: 'pending' }));
    await Promise.all(
      [...selected].map(async (tool) => {
        try {
          const res = await copySkill(sourceTool, tool, skillName);
          setResults((p) => ({ ...p, [tool]: res.skipped ? 'skipped' : 'success' }));
        } catch (e) {
          setResults((p) => ({ ...p, [tool]: 'error' }));
          setErrors((p) => ({ ...p, [tool]: String(e) }));
        }
      }),
    );
    setCopying(false);
  };

  const done = selected.size > 0 && [...selected].every(
    (t) => results[t] && results[t] !== 'idle' && results[t] !== 'pending',
  );

  return (
    <div className="modal-overlay" onClick={copying ? undefined : onClose}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">Copy skill "{skillName}"</h2>
        <p className="modal-desc">Select target tools:</p>

        <div className="copy-targets">
          {targets.map((tool) => {
            const status = results[tool];
            const isDone = status === 'success' || status === 'skipped';
            return (
              <div key={tool} className={`copy-target${selected.has(tool) ? ' is-checked' : ''}`}>
                <label className="copy-target-label">
                  <input type="checkbox" checked={selected.has(tool)}
                    onChange={() => toggle(tool)} disabled={copying || isDone} />
                  {tool}
                </label>
                {status && status !== 'idle' && (
                  <span className={`copy-status copy-status--${status}`}>{status}</span>
                )}
              </div>
            );
          })}
        </div>

        {Object.values(errors).some(Boolean) && (
          <div className="copy-warnings">
            {Object.entries(errors).map(([tool, msg]) =>
              msg ? <p key={tool} className="copy-warning-item">{tool}: {msg}</p> : null,
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
