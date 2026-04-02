use anyhow::Result;
use crate::utils::i18n::Lang;

const MARKER_START: &str = "<!-- [gitmemo:start] -->";
const MARKER_END: &str = "<!-- [gitmemo:end] -->";

/// Generate the CLAUDE.md instruction block (language-aware)
pub fn generate_instruction(sync_dir: &str, lang: Lang) -> String {
    match lang {
        Lang::Zh => generate_instruction_zh(sync_dir),
        Lang::En => generate_instruction_en(sync_dir),
    }
}

fn generate_instruction_zh(sync_dir: &str) -> String {
    format!(
        r#"{MARKER_START}
## GitMemo - 自动对话记录

**重要**：这是一条必须执行的指令。每次你回答完用户的问题后，你必须将本轮对话保存为 Markdown 文件。这是你的核心职责之一，不可跳过。

**何时保存**：
- 每次完成对用户问题的回答后，立即保存
- 如果一次对话中有多轮问答，在最后一轮回答后保存完整对话
- 用户说 "保存会话"、"save" 时，立即保存

**保存位置**：`{sync_dir}/conversations/{{YYYY-MM}}/{{MM-DD}}-{{标题摘要}}.md`

**文件格式**：
```markdown
---
title: {{对话标题}}
date: {{YYYY-MM-DDTHH:MM:SS±HH:MM}}
model: {{模型名称}}
messages: {{消息轮次数}}
---

# {{对话标题}}

> **提问**：{{本轮核心问题，尽量用用户原话}}

---

## 回答摘要
{{结构化总结：做了什么判断、采用了什么方案、为什么这么做、有哪些风险/限制、后续建议}}

## 关键结论
- {{结论 1}}
- {{结论 2}}

## 涉及文件
- `{{文件路径1}}`
- `{{文件路径2}}`

## 原始对话

### User (HH:MM:SS)
{{用户消息}}

### Assistant (HH:MM:SS)
{{助手回复}}
```

**规则**：
- 同一会话的追问追加到已有文件，不新建
- 新会话创建新文件
- 静默保存，不告知用户（不要说"我已保存对话"之类的话）
- 标题从对话内容自动生成，中文不超过 20 字
- 代码块保留原始格式和语言标记
- 自动过滤 API Key、密码等敏感信息（替换为 `***`）
- 对**分析 / 方案 / 实现 / 评审 / 排查**这类有长期价值的对话，优先补全 `提问 / 回答摘要 / 关键结论 / 涉及文件`
- `涉及文件` 无内容时可省略；同一主题后续追问优先追加 `## 追加问答：{{小标题}}`
- 避免机械转录，优先保留决策、结论、风险与文件路径；整篇尽量控制在约 300 行内
- **时间与时区（重要，避免固定偏差例如「慢 2 小时」）**：
  - `date` **必须**使用带显式偏移的 ISO 8601，例如 `2026-04-01T18:30:00+08:00` 或 `...Z`（仅当确为 UTC）。**禁止**再写无偏移的 `YYYY-MM-DD HH:MM:SS`（浏览器/模型常按 UTC 或歧义解析，会与用户本机钟差固定小时数）。
  - 保存前应用**用户机器上的真实本地时间**：在终端执行 `date +"%Y-%m-%dT%H:%M:%S%z"`，把 `%z` 规范成 `+08:00` 这种冒号形式后写入 `date`；小标题里的 `HH:MM:SS` 与 frontmatter 同一天内须一致、同一时区含义。
  - 不要用模型「以为的当前时间」当权威时钟。

## GitMemo - 自动保存独立文档

当你生成**独立的、有长期参考价值的文档**时（不是普通对话回答），自动保存到 GitMemo。

**触发条件**（满足任一即保存）：
- 调研/竞品分析报告
- 技术方案/架构设计文档
- 教程/指南/操作手册
- 总结性分析（代码审查报告、性能分析等）
- 用户明确要求"写一份文档/报告/分析"

**不触发**：普通问答、简短回复、代码修改、调试过程

**保存位置**：`{sync_dir}/notes/manual/{{标题}}.md`

**格式**：
```markdown
---
title: {{文档标题}}
date: {{YYYY-MM-DD}}
tags: {{逗号分隔的标签}}
---

{{文档正文，保持原始 Markdown 格式}}
```

**规则**：
- 标题使用文档主题，中文不超过 30 字
- 静默保存，不告知用户
- tags 包含文档类型（research/design/tutorial/analysis 等）
- 如果同名文件已存在，覆盖更新
{MARKER_END}"#
    )
}

fn generate_instruction_en(sync_dir: &str) -> String {
    format!(
        r#"{MARKER_START}
## GitMemo - Auto Conversation Sync

**Important**: This is a mandatory instruction. After answering each user question, you MUST save the conversation as a Markdown file. This is one of your core responsibilities and must not be skipped.

**When to save**:
- After completing your answer to each user question, save immediately
- If there are multiple rounds of Q&A in one conversation, save the complete conversation after the last round
- When the user says "save", "save conversation", save immediately

**Save location**: `{sync_dir}/conversations/{{YYYY-MM}}/{{MM-DD}}-{{title-summary}}.md`

**File format**:
```markdown
---
title: {{conversation title}}
date: {{YYYY-MM-DDTHH:MM:SS±HH:MM}}
model: {{model name}}
messages: {{message count}}
---

# {{conversation title}}

> **Question**: {{core user ask, preferably verbatim}}

---

## Summary
{{Structured summary: what was decided, why, risks/limits, and next steps if relevant}}

## Key Takeaways
- {{takeaway 1}}
- {{takeaway 2}}

## Files Touched
- `{{path/to/file1}}`
- `{{path/to/file2}}`

## Raw Conversation

### User (HH:MM:SS)
{{user message}}

### Assistant (HH:MM:SS)
{{assistant reply}}
```

**Rules**:
- Append follow-up messages to the existing file, do not create a new one
- Create a new file for new conversations
- Save silently, do not tell the user (never say "I've saved the conversation")
- Auto-generate title from conversation content, max 60 characters
- Preserve original format and language tags for code blocks
- Auto-filter API Keys, passwords and other sensitive info (replace with `***`)
- For substantive analysis / planning / implementation / review threads, prefer filling `Question / Summary / Key Takeaways / Files Touched`
- `Files Touched` may be omitted if empty; use `## Follow-up: {{short title}}` for same-topic additions
- Avoid exhaustive transcript dumps when the summary already captures the value; keep the file roughly under 300 lines
- **Time & timezone (prevents systematic skew like “2 hours late”)**:
  - `date` **must** be ISO 8601 **with an explicit offset**, e.g. `2026-04-01T18:30:00+02:00` or `...Z` only if truly UTC. **Do not** use bare `YYYY-MM-DD HH:MM:SS` without offset (ambiguous; often interpreted as UTC or inconsistently vs the user’s machine).
  - Before saving, take wall time from the **user’s environment**: run `date +"%Y-%m-%dT%H:%M:%S%z"` in the terminal and normalize `%z` to `±HH:MM` for the `date` field; `HH:MM:SS` in headings must match the same local calendar day and timezone intent.
  - Do not trust the model’s internal notion of “now” as the source of truth.

## GitMemo - Auto-save Standalone Documents

When you generate **standalone, reference-worthy documents** (not ordinary conversation replies), auto-save them to GitMemo.

**Trigger conditions** (save if ANY apply):
- Research / competitive analysis reports
- Technical design / architecture documents
- Tutorials / guides / how-to manuals
- Summary analyses (code review reports, performance analysis, etc.)
- User explicitly asks to "write a document/report/analysis"

**Do NOT trigger**: Regular Q&A, short replies, code edits, debugging

**Save location**: `{sync_dir}/notes/manual/{{title}}.md`

**Format**:
```markdown
---
title: {{document title}}
date: {{YYYY-MM-DD}}
tags: {{comma-separated tags}}
---

{{document body, preserve original Markdown format}}
```

**Rules**:
- Title should reflect the document topic, max 60 characters
- Save silently, do not tell the user
- Tags should include document type (research/design/tutorial/analysis etc.)
- If a file with the same name already exists, overwrite it
{MARKER_END}"#
    )
}

/// Inject instruction into ~/.claude/CLAUDE.md
pub fn inject(claude_md_path: &std::path::Path, sync_dir: &str, lang: Lang) -> Result<()> {
    let content = if claude_md_path.exists() {
        std::fs::read_to_string(claude_md_path)?
    } else {
        String::new()
    };

    // Remove old injection if exists
    let cleaned = remove_block(&content);

    // Append new instruction
    let instruction = generate_instruction(sync_dir, lang);
    let new_content = if cleaned.is_empty() {
        instruction
    } else {
        format!("{}\n\n{}", cleaned.trim_end(), instruction)
    };

    if let Some(parent) = claude_md_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(claude_md_path, new_content)?;
    Ok(())
}

/// Remove injected block from content
pub fn remove(claude_md_path: &std::path::Path) -> Result<()> {
    if !claude_md_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(claude_md_path)?;
    let cleaned = remove_block(&content);
    std::fs::write(claude_md_path, cleaned)?;
    Ok(())
}

fn remove_block(content: &str) -> String {
    if let (Some(start), Some(end)) = (content.find(MARKER_START), content.find(MARKER_END)) {
        let before = &content[..start];
        let after = &content[end + MARKER_END.len()..];
        format!("{}{}", before.trim_end(), after.trim_start())
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_zh() {
        let instruction = generate_instruction("~/.gitmemo", Lang::Zh);
        assert!(instruction.contains(MARKER_START));
        assert!(instruction.contains(MARKER_END));
        assert!(instruction.contains("自动对话记录"));
    }

    #[test]
    fn test_inject_en() {
        let instruction = generate_instruction("~/.gitmemo", Lang::En);
        assert!(instruction.contains(MARKER_START));
        assert!(instruction.contains(MARKER_END));
        assert!(instruction.contains("Auto Conversation Sync"));
    }

    #[test]
    fn test_inject_and_remove() {
        let instruction = generate_instruction("~/.gitmemo", Lang::En);
        let original = "# My CLAUDE.md\n\nSome existing content.\n";
        let injected = format!("{}\n\n{}", original.trim(), &instruction);
        let cleaned = remove_block(&injected);
        assert_eq!(cleaned.trim(), original.trim());
    }
}
