# Proton VPN TUI - Roadmap

## Completed Phases

### Phase 1: Project Setup ✅
- [x] 1.1 Rustプロジェクト初期化
- [x] 1.2 依存関係追加
- [x] 1.3 CI/CD設定 (GitHub Actions, PR時のみ)
- [x] 1.4 Linter/Formatter設定

### Phase 2: TUI Skeleton ✅
- [x] 2.1 ウィンドウ初期化・メインループ実装
- [x] 2.2 パネルレイアウト設計 (ヘッダー/メイン)
- [x] 2.3 基本スタイル定義
- [x] 2.4 ratatui導入
- [x] 2.5 vim風keybind

### Phase 3: VPN Backend Integration ✅
- [x] 3.1 protonvpn-cliラッパー作成
- [x] 3.2 サーバー一覧取得・キャッシュ
- [x] 3.3 接続状態取得 (proton0インターフェース)
- [x] 3.4 エラー処理・ログ出力
- [x] 3.5 通知ポップアップシステム
- [x] 3.6 Fuzzy検索対応 ("japan" → "JP")

---

## Current Status

### MVP Features (Implemented)
- ✅ VPN接続/切断
- ✅ サーバーリスト表示
- ✅ Fuzzy検索
- ✅ vim風キーバインド (j/k, gg/G, Ctrl+d/u)
- ✅ ヘッダー表示 (接続状態 + IP)
- ✅ サーバーキャッシュ (起動時読込)
- ✅ 'r'でリスト更新

### Remaining Features

#### Phase 4: Connection Management UI
- [ ] 4.1 サーバーリスト表示 (改良)
  - [ ] 4.1.1 国別グループ化
  - [ ] 4.1.2 都市・サーバー番号表示
  - [ ] 4.1.3 Ping表示・負荷状況
- [ ] 4.2 統計ビュー (btop風)
- [ ] 4.3 設定画面

#### Phase 5: Security Features
- [ ] 5.1 Kill Switch機能
- [ ] 5.2 Secure Core設定
- [ ] 5.3 Always On

#### Phase 6: Polish
- [ ] 6.1 テーマ対応 (ライト/ダーク)
- [ ] 6.2 国際化 (i18n)
- [ ] 6.3 macOS/Windows対応

---

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | 下一項目 |
| `k` / `↑` | 上一項目 |
| `gg` | 先頭へ |
| `G` | 末尾へ |
| `Ctrl+d` | ページダウン |
| `Ctrl+u` | ページアップ |
| `c` | 接続 |
| `d` | 切断 |
| `r` | サーバーリスト更新 |
| `/` | 検索開始 |
| `?` | ヘルプ |
| `q` | 終了 |

---

## Tech Stack

- **Language**: Rust
- **TUI Library**: ratatui
- **VPN接続**: protonvpn-cli
- **Async**: tokio

---

## Directory Structure

```
src/
├── main.rs           # Entry point
├── ui/
│   ├── app.rs        # Main TUI logic
│   └── ...
├── vpn/
│   ├── client.rs     # protonvpn-cli wrapper
│   ├── state.rs      # Connection state
│   ├── cache.rs      # Server cache
│   └── types.rs      # Data types
├── state/
│   ├── app_state.rs  # App state
│   └── server_filter.rs
└── config/
    └── settings.rs   # Settings
```
