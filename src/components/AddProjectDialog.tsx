import { useState } from 'react';
import { useConfigStore } from '../stores/configStore';

interface Props {
  onClose: () => void;
  onAdded: () => void;
}

export function AddProjectDialog({ onClose, onAdded }: Props) {
  const [path, setPath] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const { addProject } = useConfigStore();

  const handleSubmit = async () => {
    if (submitting) return;
    const trimmed = path.trim();
    if (!trimmed) {
      setError('Path cannot be empty');
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await addProject(trimmed);
      onAdded();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">Add Project</h2>
        <p className="modal-desc">
          Enter the absolute path to a project directory.
        </p>
        <div className="form-group">
          <label>Project Path</label>
          <input
            type="text"
            value={path}
            onChange={(e) => setPath(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && !submitting && handleSubmit()}
            placeholder="/home/user/my-project"
            className={error ? 'error' : ''}
            autoFocus
          />
          {error && <span className="error-msg">{error}</span>}
        </div>
        <div className="form-actions">
          <div className="form-actions-left" />
          <div className="form-actions-right">
            <button type="button" className="btn-secondary" onClick={onClose} disabled={submitting}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-primary"
              onClick={handleSubmit}
              disabled={submitting}
            >
              {submitting ? 'Adding...' : 'Add'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
