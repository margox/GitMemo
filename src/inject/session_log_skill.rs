//! Agent skill `gitmemo-session-log`: substantive Q&A summaries under `<sync>/Doc/会话记录/`.
//! Installed to both `~/.cursor/skills/` and `~/.claude/skills/` during `gitmemo init`.

use anyhow::Result;
use crate::utils::i18n::Lang;

/// Relative path inside the GitMemo sync repository (tracked by Git).
pub const SESSION_LOG_REL_DIR: &str = "Doc/会话记录";

fn skill_zh(sync_dir: &str) -> String {
    format!(
        r#"---
name: gitmemo-session-log
description: 将每次有实质内容的问答摘要写入 GitMemo 同步目录下的 Doc/会话记录/（与当前打开的项目根目录无关）。在完成分析、方案、实现或讨论类回复后主动执行；简短确认类对话可跳过。适用于 Claude Code、Cursor 等支持 Agent Skills 的环境。
---

# GitMemo 会话摘要（Q&A 归档）

## 何时写入

在完成**有实质内容**的助手回复后，将本轮问答写入 Markdown。可跳过：

- 纯确认（「好的」「OK」）
- 仅执行工具、无讨论（例如只跑一次 git status）
- 同一话题的追问：应**追加**到已有文件，而非新建

## 保存目录（绝对路径）

**必须**写入以下目录（已随 `gitmemo init` 创建），不要使用当前工作区根下的 `Doc/`：

`{sync_dir}/{rel_dir}/`

若目录不存在则创建。

## 文件命名

`{{NN}}-{{主题简述}}.md`

- `NN`：两位序号，查看该目录已有文件后递增
- 主题简述：中文，不超过约 20 字

## 文件结构

```markdown
# {{标题}}

> **提问**：{{用户原话}}

---

## 回答摘要

{{结构化摘要：表格、列表、小节}}

## 关键结论

- …

## 涉及文件

{{如有：讨论或修改过的路径}}
```

## 其它规则

1. 同一话题多轮对话可合并进同一文件
2. 单文件建议不超过约 300 行，抓要点即可
3. **不要**在回复里声明「我要保存会话记录」——静默写入即可
4. 在回复末尾、其它工具调用完成之后执行 Write；路径使用上面的**绝对路径**
5. 若已启用 GitMemo MCP（Claude Code、Cursor 等），写入后调用 `cds_sync` 提交并推送

## GitMemo 同步目录常量

本技能绑定的根目录为：`{sync_dir}`
"#,
        sync_dir = sync_dir,
        rel_dir = SESSION_LOG_REL_DIR,
    )
}

fn skill_en(sync_dir: &str) -> String {
    format!(
        r#"---
name: gitmemo-session-log
description: After substantive Q&A (analysis, plans, implementation, discussion), save a concise markdown summary under the GitMemo sync directory at Doc/会话记录/ (not under the open project's root). Skip trivial confirmations. Works in Claude Code, Cursor, and other environments with Agent Skills.
---

# GitMemo session log (Q&A archive)

## When to write

After a **substantive** assistant turn, write a markdown record. Skip:

- Trivial confirmations ("OK", "sure")
- Tool-only turns with no real discussion

For follow-ups on the **same topic**, **append** to the existing file instead of creating a new one.

## Save directory (absolute path)

**Always** write under (created by `gitmemo init`):

`{sync_dir}/{rel_dir}/`

Do **not** use `Doc/` at the current workspace root.

## File naming

`{{NN}}-{{short-topic}}.md` — two-digit sequence + short topic (Chinese ok).

## File structure

```markdown
# {{Title}}

> **Question**: {{verbatim user question}}

---

## Summary

{{Structured summary}}

## Key takeaways

- …

## Files touched

{{If any}}
```

## Rules

1. Batch related Q&A into one file when it is one thread
2. Keep each file under ~300 lines
3. Do **not** announce that you are saving — write silently
4. Use the **absolute path** above; run Write as the **last** step of the response
5. If GitMemo MCP is enabled (Claude Code, Cursor, etc.), call `cds_sync` after writing

## GitMemo sync root

`{sync_dir}`
"#,
        sync_dir = sync_dir,
        rel_dir = SESSION_LOG_REL_DIR,
    )
}

/// Full `SKILL.md` content (same file installed under Cursor and Claude Code skill dirs).
pub fn generate(sync_dir: &str, lang: Lang) -> String {
    match lang {
        Lang::Zh => skill_zh(sync_dir),
        Lang::En => skill_en(sync_dir),
    }
}

pub fn install(skill_dir: &std::path::Path, sync_dir: &str, lang: Lang) -> Result<()> {
    std::fs::create_dir_all(skill_dir)?;
    std::fs::write(skill_dir.join("SKILL.md"), generate(sync_dir, lang))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_sync_dir_and_target_folder() {
        let s = generate("/home/u/.gitmemo", Lang::En);
        assert!(s.contains("/home/u/.gitmemo"));
        assert!(s.contains(SESSION_LOG_REL_DIR));
        assert!(s.contains("gitmemo-session-log"));
        assert!(s.contains("cds_sync"));
    }

    #[test]
    fn zh_contains_chinese_hints() {
        let s = generate("~/.gitmemo", Lang::Zh);
        assert!(s.contains("会话摘要"));
    }
}
