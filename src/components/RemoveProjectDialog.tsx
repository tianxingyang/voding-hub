import { useState } from 'react';
import { useConfigStore, type Project } from '../stores/configStore';

interface Props {
  project: Project;
  onClose: () => void;
  onRemoved: () => void;
}

export function RemoveProjectDialog({ project, onClose, onRemoved }: Props) {
  const [removing, setRemoving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { removeProject } = useConfigStore();

  const handleRemove = async () => {
    if (removing) return;
    setRemoving(true);
    setError(null);
    try {
      await removeProject(project.id);
      onRemoved();
      onClose();
    } catch (e) {
      setRemoving(false);
      setError(String(e));
    }
  };

  return (
    <div className="modal-overlay" onClick={removing ? undefined : onClose}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">Remove Project</h2>
        <p className="modal-desc">
          Remove <strong>{project.name}</strong> from the list?
          This will not delete any files on disk.
        </p>
        {error && <p className="error-msg">{error}</p>}
        <div className="form-actions">
          <div className="form-actions-left" />
          <div className="form-actions-right">
            <button type="button" className="btn-secondary" onClick={onClose} disabled={removing}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-danger"
              onClick={handleRemove}
              disabled={removing}
            >
              {removing ? 'Removing...' : 'Remove'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
