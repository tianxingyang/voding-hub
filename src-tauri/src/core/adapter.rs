use crate::core::{ConfigScope, McpServer, Skill};
use std::path::PathBuf;

pub trait ConfigAdapter {
    fn tool_name(&self) -> &'static str;
    fn global_config_path(&self) -> PathBuf;
    fn project_config_path(&self, project: &PathBuf) -> PathBuf;
    fn read_mcp_servers(&self, scope: &ConfigScope) -> Result<Vec<McpServer>, String>;
    fn write_mcp_server(&self, server: &McpServer, scope: &ConfigScope) -> Result<(), String>;
    fn delete_mcp_server(&self, name: &str, scope: &ConfigScope) -> Result<(), String>;
    fn read_skills(&self, scope: &ConfigScope) -> Result<Vec<Skill>, String>;
    fn write_skill(&self, skill: &Skill, scope: &ConfigScope) -> Result<(), String>;
    fn delete_skill(&self, name: &str, scope: &ConfigScope) -> Result<(), String>;
    fn read_rules(&self, scope: &ConfigScope) -> Result<String, String>;
    fn write_rules(&self, content: &str, scope: &ConfigScope) -> Result<(), String>;
}
