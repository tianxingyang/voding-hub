import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export const TOOLS = ['ClaudeCode', 'Codex', 'Gemini', 'OpenCode'] as const;
export type ToolType = (typeof TOOLS)[number];

export interface McpServer {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  url?: string;
  enabled: boolean;
}

export interface Skill {
  name: string;
  description?: string;
  content: string;
  path: string;
}

export interface Project {
  id: number;
  name: string;
  path: string;
  tools: ToolType[];
  created_at: number;
  updated_at: number;
}

interface CopyResult {
  server: McpServer | null;
  warnings: string[];
  skipped: boolean;
}

interface SkillCopyResult {
  skipped: boolean;
}

export interface ProjectConfigSummary {
  tool: ToolType;
  mcp_count: number;
  skills_count: number;
  has_rules: boolean;
}

type ToolRecord<T> = Record<ToolType, T>;

interface ConfigState {
  mcpServers: ToolRecord<McpServer[]>;
  skills: ToolRecord<Skill[]>;
  rules: ToolRecord<string>;
  projects: Project[];
  currentProjectPath: string | null;
  loading: { mcp: number; skills: number; rules: number; projects: number };
  error: string | null;
}

interface ConfigActions {
  setCurrentProject: (path: string | null) => void;
  clearError: () => void;
  // Projects
  fetchProjects: () => Promise<void>;
  addProject: (path: string) => Promise<Project>;
  removeProject: (id: number) => Promise<void>;
  detectProjectTools: (path: string) => Promise<ToolType[]>;
  getProjectConfigSummary: (path: string) => Promise<ProjectConfigSummary[]>;
  // MCP
  fetchMcpServers: (tool: ToolType) => Promise<void>;
  fetchAllMcpServers: () => Promise<void>;
  saveMcpServer: (tool: ToolType, server: McpServer) => Promise<void>;
  deleteMcpServer: (tool: ToolType, name: string) => Promise<void>;
  copyMcpServer: (from: ToolType, to: ToolType, name: string) => Promise<CopyResult>;
  // Skills
  fetchSkills: (tool: ToolType) => Promise<void>;
  fetchAllSkills: () => Promise<void>;
  saveSkill: (tool: ToolType, skill: Skill) => Promise<void>;
  deleteSkill: (tool: ToolType, name: string) => Promise<void>;
  copySkill: (from: ToolType, to: ToolType, name: string) => Promise<SkillCopyResult>;
  // Rules
  fetchRules: (tool: ToolType) => Promise<void>;
  fetchAllRules: () => Promise<void>;
  saveRules: (tool: ToolType, content: string) => Promise<void>;
  // Events
  setupEventListeners: () => Promise<UnlistenFn>;
}

const emptyToolRecord = <T>(factory: () => T): ToolRecord<T> => ({
  ClaudeCode: factory(), Codex: factory(), Gemini: factory(), OpenCode: factory(),
});

let eventListenerPromise: Promise<UnlistenFn> | null = null;
let eventListenerRefs = 0;

export const useConfigStore = create<ConfigState & ConfigActions>((set, get) => ({
  mcpServers: emptyToolRecord(() => []),
  skills: emptyToolRecord(() => []),
  rules: emptyToolRecord(() => ''),
  projects: [],
  currentProjectPath: null,
  loading: { mcp: 0, skills: 0, rules: 0, projects: 0 },
  error: null,

  setCurrentProject: (path) => set({ currentProjectPath: path }),
  clearError: () => set({ error: null }),

  fetchProjects: async () => {
    set((s) => ({ loading: { ...s.loading, projects: s.loading.projects + 1 }, error: null }));
    try {
      const projects = await invoke<Project[]>('list_projects');
      set({ projects });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set((s) => ({ loading: { ...s.loading, projects: s.loading.projects - 1 } }));
    }
  },

  addProject: async (path) => {
    set((s) => ({ loading: { ...s.loading, projects: s.loading.projects + 1 }, error: null }));
    try {
      const project = await invoke<Project>('add_project', { path });
      set((s) => ({ projects: [...s.projects, project] }));
      return project;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    } finally {
      set((s) => ({ loading: { ...s.loading, projects: s.loading.projects - 1 } }));
    }
  },

  removeProject: async (id) => {
    set((s) => ({ projects: s.projects.filter((p) => p.id !== id), error: null }));
    try {
      await invoke('remove_project', { id });
    } catch (e) {
      await get().fetchProjects();
      set({ error: String(e) });
      throw e;
    }
  },

  detectProjectTools: async (path) => {
    return invoke<ToolType[]>('detect_project_tools', { path });
  },

  getProjectConfigSummary: async (path) => {
    return invoke<ProjectConfigSummary[]>('get_project_config_summary', { path });
  },

  fetchMcpServers: async (tool) => {
    const scopePath = get().currentProjectPath;
    set((s) => ({ loading: { ...s.loading, mcp: s.loading.mcp + 1 }, error: null }));
    try {
      const servers = await invoke<McpServer[]>('get_mcp_servers', {
        tool, projectPath: scopePath,
      });
      if (get().currentProjectPath === scopePath) {
        set((s) => ({ mcpServers: { ...s.mcpServers, [tool]: servers } }));
      }
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set((s) => ({ loading: { ...s.loading, mcp: s.loading.mcp - 1 } }));
    }
  },

  fetchAllMcpServers: async () => {
    await Promise.all(TOOLS.map((t) => get().fetchMcpServers(t)));
  },

  saveMcpServer: async (tool, server) => {
    const scopePath = get().currentProjectPath;
    const prev = get().mcpServers[tool];
    const updated = prev.some((s) => s.name === server.name)
      ? prev.map((s) => (s.name === server.name ? server : s))
      : [...prev, server];
    set((s) => ({ mcpServers: { ...s.mcpServers, [tool]: updated }, error: null }));
    try {
      await invoke('save_mcp_server', { tool, server, projectPath: scopePath });
    } catch (e) {
      if (get().currentProjectPath === scopePath) {
        await get().fetchMcpServers(tool);
      }
      set({ error: String(e) });
      throw e;
    }
  },

  deleteMcpServer: async (tool, name) => {
    const scopePath = get().currentProjectPath;
    const prev = get().mcpServers[tool];
    set((s) => ({
      mcpServers: { ...s.mcpServers, [tool]: prev.filter((x) => x.name !== name) },
      error: null,
    }));
    try {
      await invoke('delete_mcp_server', { tool, name, projectPath: scopePath });
    } catch (e) {
      if (get().currentProjectPath === scopePath) {
        await get().fetchMcpServers(tool);
      }
      set({ error: String(e) });
      throw e;
    }
  },

  copyMcpServer: async (from, to, name) => {
    const scopePath = get().currentProjectPath;
    try {
      const result = await invoke<CopyResult>('copy_mcp_to_tool', {
        fromTool: from, toTool: to, serverName: name, projectPath: scopePath,
      });
      if (!result.skipped && result.server && get().currentProjectPath === scopePath) {
        set((s) => {
          const exists = s.mcpServers[to].some((x) => x.name === result.server!.name);
          if (exists) return s;
          return { mcpServers: { ...s.mcpServers, [to]: [...s.mcpServers[to], result.server!] } };
        });
      }
      return result;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  fetchSkills: async (tool) => {
    const scopePath = get().currentProjectPath;
    set((s) => ({ loading: { ...s.loading, skills: s.loading.skills + 1 }, error: null }));
    try {
      const skills = await invoke<Skill[]>('get_skills', {
        tool, projectPath: scopePath,
      });
      if (get().currentProjectPath === scopePath) {
        set((s) => ({ skills: { ...s.skills, [tool]: skills } }));
      }
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set((s) => ({ loading: { ...s.loading, skills: s.loading.skills - 1 } }));
    }
  },

  fetchAllSkills: async () => {
    await Promise.all(TOOLS.map((t) => get().fetchSkills(t)));
  },

  saveSkill: async (tool, skill) => {
    const scopePath = get().currentProjectPath;
    const prev = get().skills[tool];
    const updated = prev.some((s) => s.name === skill.name)
      ? prev.map((s) => (s.name === skill.name ? skill : s))
      : [...prev, skill];
    set((s) => ({ skills: { ...s.skills, [tool]: updated }, error: null }));
    try {
      await invoke('save_skill', { tool, skill, projectPath: scopePath });
    } catch (e) {
      if (get().currentProjectPath === scopePath) {
        await get().fetchSkills(tool);
      }
      set({ error: String(e) });
      throw e;
    }
  },

  deleteSkill: async (tool, name) => {
    const scopePath = get().currentProjectPath;
    const prev = get().skills[tool];
    set((s) => ({
      skills: { ...s.skills, [tool]: prev.filter((x) => x.name !== name) },
      error: null,
    }));
    try {
      await invoke('delete_skill', { tool, name, projectPath: scopePath });
    } catch (e) {
      if (get().currentProjectPath === scopePath) {
        await get().fetchSkills(tool);
      }
      set({ error: String(e) });
      throw e;
    }
  },

  copySkill: async (from, to, name) => {
    const scopePath = get().currentProjectPath;
    try {
      const result = await invoke<SkillCopyResult>('copy_skill_to_tool', {
        fromTool: from, toTool: to, skillName: name, projectPath: scopePath,
      });
      if (!result.skipped && get().currentProjectPath === scopePath) {
        await get().fetchSkills(to);
      }
      return result;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  fetchRules: async (tool) => {
    const scopePath = get().currentProjectPath;
    set((s) => ({ loading: { ...s.loading, rules: s.loading.rules + 1 }, error: null }));
    try {
      const content = await invoke<string>('get_rules', {
        tool, projectPath: scopePath,
      });
      if (get().currentProjectPath === scopePath) {
        set((s) => ({ rules: { ...s.rules, [tool]: content } }));
      }
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set((s) => ({ loading: { ...s.loading, rules: s.loading.rules - 1 } }));
    }
  },

  fetchAllRules: async () => {
    await Promise.all(TOOLS.map((t) => get().fetchRules(t)));
  },

  saveRules: async (tool, content) => {
    const scopePath = get().currentProjectPath;
    set((s) => ({ rules: { ...s.rules, [tool]: content }, error: null }));
    try {
      await invoke('save_rules', { tool, content, projectPath: scopePath });
    } catch (e) {
      if (get().currentProjectPath === scopePath) {
        await get().fetchRules(tool);
      }
      set({ error: String(e) });
      throw e;
    }
  },

  setupEventListeners: async () => {
    eventListenerRefs++;
    if (!eventListenerPromise) {
      eventListenerPromise = listen<{ tool: string; kind: string }>('config-changed', (event) => {
        const { tool, kind } = event.payload;
        if (!TOOLS.includes(tool as ToolType)) return;
        const t = tool as ToolType;
        if (kind === 'mcp') get().fetchMcpServers(t);
        else if (kind === 'skills') get().fetchSkills(t);
        else if (kind === 'rules') get().fetchRules(t);
      });
    }
    try {
      const unlisten = await eventListenerPromise;
      return () => {
        eventListenerRefs--;
        if (eventListenerRefs <= 0) {
          unlisten();
          eventListenerPromise = null;
          eventListenerRefs = 0;
        }
      };
    } catch {
      eventListenerRefs--;
      eventListenerPromise = null;
      return () => {};
    }
  },
}));
