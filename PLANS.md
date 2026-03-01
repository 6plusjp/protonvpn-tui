# Proton VPN TUI

## Goal

Create a fast, keyboard-driven Terminal UI for ProtonVPN.

## Current Status

### MVP: Released ✅
- VPN接続/切断 via protonvpn-cli
- サーバーリスト表示 + Fuzzy検索
- vim風キーバインド
- 接続状態 + IP表示
- サーバーキャッシュ

### Tech Stack

- **Language**: Rust
- **TUI Library**: ratatui
- **VPN**: protonvpn-cli
- **Async**: tokio
- **Serialization**: serde, toml

## Features

### Connection Management
- VPN接続/切断
- サーバー選択 (国 > 都市 > サーバー番号)
- 接続状態表示 (real-time IP)

### UI/UX
- vim風キーバインド (j/k/h/l, gg/G, /検索, q終了)
- Fuzzy検索 ("japan" → "JP")
- 通知ポップアップ
- サーバーキャッシュ (高速起動)

### Future Features
- 統計ビュー (btop風)
- Kill Switch
- Secure Core
- テーマ対応

## Requirements

- Linux with `proton0` network interface
- protonvpn-cli installed

## Distribution

- cargo install対応予定
