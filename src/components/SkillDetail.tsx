import { useState, useMemo } from 'react';
import type { Skill, ToolType } from '../stores/configStore';
import { validateSkill } from './skillValidation';

interface SkillDetailProps {
  skill: Skill;
  tool: ToolType;
  onDelete: () => void;
  onCopy: () => void;
  onClose: () => void;
}

function parseFrontmatter(content: string): { frontmatter: string; body: string } {
  if (!content.startsWith('---\n') && !content.startsWith('---\r\n')) {
    return { frontmatter: '', body: content };
  }
  const end = content.indexOf('\n---', 3);
  if (end === -1) return { frontmatter: '', body: content };
  const fmEnd = content.indexOf('\n', end + 1);
  return {
    frontmatter: content.slice(0, fmEnd === -1 ? undefined : fmEnd),
    body: fmEnd === -1 ? '' : content.slice(fmEnd + 1),
  };
}

export function SkillDetail({ skill, onDelete, onCopy, onClose }: SkillDetailProps) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const { frontmatter, body } = useMemo(() => parseFrontmatter(skill.content), [skill.content]);
  const validation = useMemo(() => validateSkill(skill), [skill]);

  return (
    <div className="skill-detail">
      <h3 className="skill-detail-name">{skill.name}</h3>
      {skill.description && (
        <p className="skill-detail-desc">{skill.description}</p>
      )}
      <p className="skill-detail-path" title={skill.path}>{skill.path}</p>

      <div className={`skill-validation-badge${validation.valid ? ' is-valid' : ' is-invalid'}`}>
        {validation.valid ? 'Valid' : validation.errors.join(' / ')}
      </div>

      <div className="skill-content">
        {frontmatter && (
          <pre className="skill-frontmatter"><code>{frontmatter}</code></pre>
        )}
        {body && (
          <pre className="skill-body"><code>{body}</code></pre>
        )}
      </div>

      <div className="form-actions">
        <div className="form-actions-left">
          <button type="button" className="btn-secondary" onClick={onClose}>Close</button>
        </div>
        <div className="form-actions-right">
          <button type="button" className="btn-secondary" onClick={onCopy}>Copy to...</button>
          {confirmDelete ? (
            <span className="confirm-delete">
              Sure? <button type="button" className="btn-danger" onClick={onDelete}>Yes</button>
              <button type="button" className="btn-secondary" onClick={() => setConfirmDelete(false)}>No</button>
            </span>
          ) : (
            <button type="button" className="btn-danger" onClick={() => setConfirmDelete(true)}>Delete</button>
          )}
        </div>
      </div>
    </div>
  );
}
