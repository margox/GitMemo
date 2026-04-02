# GitMemo CLI i18n 独立计划

## 目标

把 CLI 的用户可见输出单独抽成一个执行流，不和 Desktop 页面体验改动混在同一批里推进。

## 覆盖盘点

- 命令入口与帮助：`src/cli/mod.rs`
- 主命令路由与输出：`src/main.rs`
- 注入文案：`src/inject/claude_md.rs`
- Cursor 规则文案：`src/inject/cursor_rules.rs`
- 统一文案出口：`src/utils/i18n.rs`

## 建议批次

### Batch 1：高频用户命令

- `init`
- `status`
- `search`
- `reindex`
- `uninstall`
- `remote`
- `branch`

交付要求：

- 成功提示、错误提示、交互提问都走 `I18n`
- 保持现有英文为默认回退
- 新增中文覆盖时，不改变命令行为和参数

### Batch 2：注入与配置文案

- `CLAUDE.md` 注入提示
- Cursor rules 注入/移除提示
- MCP 注册/失败提示
- SSH / remote 配置相关说明

交付要求：

- 注入到文件内的说明与终端提示语义保持一致
- 中英文中的路径示例、目录结构、同步语义保持一致

### Batch 3：帮助与边角错误

- clap 帮助文案
- 参数校验失败
- 文件不存在、仓库不存在、远程不可达等边角错误

交付要求：

- 仅处理用户可见文本，不引入新的控制流分支
- 对难以局部国际化的第三方原始错误，保留英文原文并在前面补中文上下文

## 实施原则

- 不在同一提交里同时改 CLI i18n 与 Desktop 交互
- 每个批次都要单独跑 `cargo check`
- 优先补“用户看得见且频率高”的输出，再处理长尾路径

## 完成标准

- CLI 主流程在 `--lang zh` 下不再出现明显混杂英文的自有文案
- 注入类提示与 README 口径一致
- Desktop 改动可以独立发布，不依赖 CLI i18n 完成
