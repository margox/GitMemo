---
name: save
description: Save the current conversation to GitMemo. Use when the user says "save", "保存会话", "save conversation", or "/save".
---

# Save Conversation to GitMemo

Save the current conversation as a Markdown file in the GitMemo repository.

## Instructions

1. Determine the sync directory: `~/.gitmemo/conversations/`
1. Generate a filename: `{YYYY-MM}/{MM-DD}-{title_summary}.md` where `title_summary` is a short Chinese description (max 20 chars) of the conversation topic.
1. Prefer this fuller format instead of a bare transcript:

```markdown
---
title: {conversation title}
date: {YYYY-MM-DDTHH:MM:SS±HH:MM}
model: {model name}
messages: {message count}
---

# {conversation title}

> **提问**：{user's core question, preferably verbatim}

---

## 回答摘要

{Structured summary of the answer: what was decided, why, what changed, risks/limits, and next steps if relevant.}

## 关键结论

- {key takeaway 1}
- {key takeaway 2}

## 涉及文件

- `{path/to/file1}`
- `{path/to/file2}`

## 原始对话

### User (HH:MM:SS)
{user message}

### Assistant (HH:MM:SS)
{assistant response}
```

1. `涉及文件` can be omitted if no files were discussed or modified.
1. If this is a follow-up to the same topic, append a new section like `## 追加问答：{short title}` to the existing file instead of creating a new file.
1. Keep the file focused and useful; do not dump repetitive content if the summary already captures it well.

**Time**: `date` must include an explicit timezone offset (ISO 8601), e.g. `2026-04-01T18:30:00+08:00`. Before writing, run `date +"%Y-%m-%dT%H:%M:%S%z"` on the user’s machine and normalize `%z` to `±HH:MM`. Do **not** use bare `YYYY-MM-DD HH:MM:SS` (ambiguous; often wrong vs local clock by a fixed number of hours). Match heading `HH:MM:SS` to the same local time intent as frontmatter `date`.

1. Include ALL messages from the current conversation in the `原始对话` section when they add value.
1. Preserve code blocks with original language tags.
1. Filter sensitive info (API keys, passwords) and replace them with `***`.
1. Match the user's language unless they explicitly asked for another language.
1. **New file**: Use the Write tool to create the file.
1. **Existing file (append mode)**:

   First use the Read tool to read the existing file content.

   Then:

   - Identify which messages are already saved by comparing message content, not timestamps.
   - Only append NEW messages that do not exist in the file yet.
   - Update `回答摘要` / `关键结论` / `涉及文件` if the new turn changed the conclusion.
   - Use the Edit tool to append or update content. Do not use Write, which overwrites the file.
   - Update the `messages` count in the frontmatter to reflect the new total.

1. Do NOT tell the user "I have saved the conversation" and save silently.
