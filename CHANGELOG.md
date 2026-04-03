# Changelog

All notable changes to GitMemo will be documented in this file.

## [Unreleased]

### Added
- **Dashboard**: Recent activity feed showing latest items across all types
- **Dashboard**: Clipboard monitoring status indicator
- **System notifications**: macOS Notification Center integration for sync failures and clipboard captures (background only)
- **Clipboard**: Larger image thumbnails in clip list with full-width preview
- **First-launch**: Unified "not initialized" screen with setup guidance for new users
- **Backend**: `get_recent_activity` command for cross-category recent items

### Fixed
- **Image paste**: Fallback to `clipboardData.files` for Tauri WKWebView compatibility
- **Clipboard**: Added initialization check to `start_clipboard_watch` to prevent crashes
- **TypeScript**: Fixed `nativeEvent.isComposing` error in QuickPaste

## [0.1.15] - 2026-03-27

### Fixed
- Align session-log skill with `conversations/YYYY-MM/` layout

## [0.1.14] - 2026-03-27

### Added
- Full-text search across clips, plans, imports, claude-config, and conversations
- Clipboard hardening and session-log skills
- Desktop UX improvements

## [0.1.13] - 2026-03-26

### Added
- Clipboard image capture with PNG encoding and local image rendering in Markdown
- Quick Paste floating window with command palette (Cmd+Shift+Space)
- System tray with Open/Sync/Clipboard/Quit menu
- Auto-start and clipboard auto-start settings
- Global shortcut Cmd+Shift+G to show and search

### Fixed
- Save skill uses Edit for append to prevent content overwrite

## [0.1.12] - 2026-03-26

### Fixed
- Update plugin homepage URL to git-memo.vercel.app

## [0.1.11] - 2026-03-25

### Added
- Claude Code plugin packaging for marketplace submission

## [0.1.10] - 2026-03-25

### Added
- Desktop app build added to CI pipeline (macOS aarch64 + x86_64)
- DMG and .app bundle packaging
