//! Agent skill `gitmemo-session-log`: substantive Q&A summaries as normal conversation markdown
//! under `conversations/{YYYY-MM}/`, same path rule as CLAUDE.md / global Cursor rule — no extra subfolders.

use anyhow::Result;
use crate::utils::i18n::Lang;

fn skill_zh(sync_dir: &str) -> String {
    format!(
        r#"---
name: gitmemo-session-log
description: 将有实质内容的问答摘要保存为 GitMemo 同步目录下的对话 Markdown，路径规则与自动保存会话完全一致（conversations/年-月/）。不要写到当前项目仓库。完成分析、方案、实现或讨论类回复后主动执行；简短确认可跳过。
---

# GitMemo 会话摘要（与对话同目录规则）

## 何时写入

在完成**有实质内容**的助手回复后写入。可跳过：纯确认、仅执行工具无讨论。同一话题的追问**追加**到已有文件。

## 保存路径（与 GitMemo 官方一致）

**必须**使用与 `CLAUDE.md` / Cursor 全局规则相同的格式（不要用项目工作区里的路径）：

`{sync_dir}/conversations/{{YYYY-MM}}/{{MM-DD}}-{{主题摘要}}.md`

- `YYYY-MM`、`MM-DD`：以用户本地日期为准（保存前可用终端 `date` 确认）
- 主题摘要：中文约 20 字以内，文件名中非法字符改为 `_`
- 若该月目录不存在则创建

## 文件内容结构（可与普通对话文件相同风格）

```markdown
---
title: {{标题}}
date: {{ISO8601 带时区偏移}}
---

# {{标题}}

> **提问**：{{用户原话}}

---

## 回答摘要
…
```

（若你更习惯与现有对话文件完全相同的 `### User` / `### Assistant` 结构，也可以。）

## 规则

1. 静默保存，不要声明「我要保存会话记录」
2. 在回复末尾、其它工具调用完成之后执行 Write；使用**绝对路径**
3. 若已启用 GitMemo MCP，写入后调用 `cds_sync`
4. **禁止**使用 `Doc/`、`会话记录/` 等额外目录；只在 `conversations/{{YYYY-MM}}/` 下创建文件

## 同步根目录

`{sync_dir}`
"#,
        sync_dir = sync_dir,
    )
}

fn skill_en(sync_dir: &str) -> String {
    format!(
        r#"---
name: gitmemo-session-log
description: Save substantive Q&A summaries as normal GitMemo conversation Markdown under conversations/{{YYYY-MM}}/ (same rule as CLAUDE.md / Cursor global rule). Not the open project repo.
---

# GitMemo session log (same path as conversations)

## When to write

After a **substantive** turn; skip trivial confirmations. Append follow-ups to the same file for one thread.

## Path (same as official GitMemo)

`{sync_dir}/conversations/{{YYYY-MM}}/{{MM-DD}}-{{title-summary}}.md`

- Use the user’s local date; create the month folder if needed
- Title slug: concise; sanitize filename characters

## Content

Use frontmatter + body similar to other conversation files, or match your existing `### User` / `### Assistant` style.

## Rules

1. Save silently
2. Write last; use **absolute** paths
3. Call `cds_sync` after write if GitMemo MCP is enabled
4. Do **not** use extra folders like `Doc/` or `会话记录/` — only `conversations/{{YYYY-MM}}/`

## Sync root

`{sync_dir}`
"#,
        sync_dir = sync_dir,
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
    fn contains_sync_dir_and_monthly_pattern() {
        let s = generate("/home/u/.gitmemo", Lang::En);
        assert!(s.contains("/home/u/.gitmemo"));
        assert!(s.contains("conversations/{YYYY-MM}"));
        assert!(s.contains("gitmemo-session-log"));
        assert!(s.contains("cds_sync"));
    }

    #[test]
    fn zh_contains_hints() {
        let s = generate("~/.gitmemo", Lang::Zh);
        assert!(s.contains("会话摘要"));
        assert!(s.contains("禁止"));
    }
}
