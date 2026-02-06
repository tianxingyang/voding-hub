mod claude;
mod codex;
mod gemini;
mod opencode;
#[cfg(test)]
mod tests;

pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use gemini::GeminiAdapter;
pub use opencode::OpenCodeAdapter;
