import { useState } from 'react';
import type { McpServer, ToolType } from '../stores/configStore';

const NAME_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/;

interface McpServerFormProps {
  server?: McpServer;
  tool: ToolType;
  isNew: boolean;
  onSave: (server: McpServer) => void;
  onDelete?: () => void;
  onCancel: () => void;
  onCopy?: () => void;
}

const blank: McpServer = { name: '', command: '', args: [], env: {}, enabled: true };

export function McpServerForm({ server, tool, isNew, onSave, onDelete, onCancel, onCopy }: McpServerFormProps) {
  const [form, setForm] = useState<McpServer>(() => server ? { ...server } : { ...blank });
  const [envPairs, setEnvPairs] = useState<[string, string][]>(
    () => server ? Object.entries(server.env) : [],
  );
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [confirmDelete, setConfirmDelete] = useState(false);

  const validate = (): boolean => {
    const e: Record<string, string> = {};
    if (!form.name) e.name = 'Required';
    else if (!NAME_RE.test(form.name)) e.name = 'Must be kebab-case';
    else if (form.name.length > 64) e.name = 'Max 64 characters';
    if (!form.command.trim()) e.command = 'Required';
    setErrors(e);
    return !Object.keys(e).length;
  };

  const handleSave = () => {
    if (!validate()) return;
    const env = Object.fromEntries(envPairs.filter(([k]) => k.trim()));
    onSave({ ...form, env, url: form.url?.trim() || undefined });
  };

  const setArg = (i: number, v: string) => {
    const next = [...form.args];
    next[i] = v;
    setForm({ ...form, args: next });
  };

  const removeArg = (i: number) =>
    setForm({ ...form, args: form.args.filter((_, idx) => idx !== i) });

  const setEnvKey = (i: number, k: string) => {
    const next = envPairs.map((pair, idx) => idx === i ? [k, pair[1]] as [string, string] : pair);
    setEnvPairs(next);
  };

  const setEnvVal = (i: number, v: string) => {
    const next = envPairs.map((pair, idx) => idx === i ? [pair[0], v] as [string, string] : pair);
    setEnvPairs(next);
  };

  return (
    <div className="mcp-form">
      <div className="form-group">
        <label>Name</label>
        <input
          type="text" value={form.name} readOnly={!isNew}
          className={errors.name ? 'error' : ''}
          placeholder="e.g. postgres-server"
          onChange={(e) => setForm({ ...form, name: e.target.value })}
        />
        {errors.name && <span className="error-msg">{errors.name}</span>}
      </div>

      <div className="form-group">
        <label>Command</label>
        <input
          type="text" value={form.command}
          className={errors.command ? 'error' : ''}
          placeholder="e.g. npx"
          onChange={(e) => setForm({ ...form, command: e.target.value })}
        />
        {errors.command && <span className="error-msg">{errors.command}</span>}
      </div>

      <div className="form-group">
        <label>Arguments</label>
        <div className="dynamic-list">
          {form.args.map((arg, i) => (
            <div key={i} className="dynamic-row">
              <input type="text" value={arg} onChange={(e) => setArg(i, e.target.value)} />
              <button type="button" className="icon-btn" onClick={() => removeArg(i)}>&times;</button>
            </div>
          ))}
          <button type="button" className="form-add-btn"
            onClick={() => setForm({ ...form, args: [...form.args, ''] })}
          >+ Add Argument</button>
        </div>
      </div>

      <div className="form-group">
        <label>Environment Variables</label>
        <div className="dynamic-list">
          {envPairs.map(([k, v], i) => (
            <div key={i} className="dynamic-row">
              <input type="text" placeholder="KEY" value={k} onChange={(e) => setEnvKey(i, e.target.value)} />
              <input type="text" placeholder="VALUE" value={v} onChange={(e) => setEnvVal(i, e.target.value)} />
              <button type="button" className="icon-btn" onClick={() => setEnvPairs(envPairs.filter((_, idx) => idx !== i))}>&times;</button>
            </div>
          ))}
          <button type="button" className="form-add-btn"
            onClick={() => setEnvPairs([...envPairs, ['', '']])}
          >+ Add Variable</button>
        </div>
      </div>

      <div className="form-group">
        <label>URL (optional)</label>
        <input type="text" value={form.url ?? ''} placeholder="e.g. http://localhost:8080"
          onChange={(e) => setForm({ ...form, url: e.target.value })} />
      </div>

      {tool === 'Codex' && (
        <div className="form-group form-checkbox">
          <label>
            <input type="checkbox" checked={form.enabled}
              onChange={(e) => setForm({ ...form, enabled: e.target.checked })} />
            Enabled
          </label>
        </div>
      )}

      <div className="form-actions">
        <div className="form-actions-left">
          <button type="button" className="btn-primary" onClick={handleSave}>{isNew ? 'Create' : 'Save'}</button>
          <button type="button" className="btn-secondary" onClick={onCancel}>Cancel</button>
        </div>
        {!isNew && (
          <div className="form-actions-right">
            {onCopy && <button type="button" className="btn-secondary" onClick={onCopy}>Copy to...</button>}
            {onDelete && (confirmDelete ? (
              <span className="confirm-delete">
                Sure? <button type="button" className="btn-danger" onClick={onDelete}>Yes</button>
                <button type="button" className="btn-secondary" onClick={() => setConfirmDelete(false)}>No</button>
              </span>
            ) : (
              <button type="button" className="btn-danger" onClick={() => setConfirmDelete(true)}>Delete</button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}