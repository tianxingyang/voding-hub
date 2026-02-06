import type { Skill } from '../stores/configStore';

const NAME_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/;

export interface SkillValidation {
  valid: boolean;
  errors: string[];
}

export function validateSkill(skill: Skill): SkillValidation {
  const errors: string[] = [];
  if (!NAME_RE.test(skill.name)) errors.push('Name must be kebab-case');
  if (!skill.content?.startsWith('---')) errors.push('Missing YAML frontmatter');
  if (!skill.description) errors.push('Missing description');
  return { valid: errors.length === 0, errors };
}
