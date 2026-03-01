# Proton VPN TUI - Roadmap

## Phase 1: Project Setup (完了)

- [ ] 1.1 Rustプロジェクト初期化 (`cargo new protonvpn-tui`)
- [ ] 1.2 依存関係追加 (crossterm, tokio, serde, etc.)
- [ ] 1.3 CI/CD設定 (GitHub Actions)
  - [ ] 1.3.1 CIワークフロー作成 (test, clippy, fmt)
  - [ ] 1.3.2 キャッシュ設定 (依存関係キャッシュ)
  - [ ] 1.3.3 マトリックス戦略 (複数OSテスト: ubuntu, macOS, Windows)
  - [ ] 1.3.4 PR自動チェック設定
- [ ] 1.4 Linter/Formatter設定 (clippy, rustfmt)

---

## Phase 2: TUI Skeleton (基盤完成)

- [ ] 2.1 ウィンドウ初期化・メインループ実装
- [ ] 2.2 パネルレイアウト設計 (ヘッダー/メイン/フッター)
- [ ] 2.3 基本スタイル定義 (配色、フォント)
- [ ] 2.4 アプリ起動時のスプラッシュ画面
- [ ] 2.5 アプリ終了処理

---

## Phase 3: VPN Backend Integration

- [ ] 3.1 protonvpn-cliラッパー作成
  - [ ] 3.1.1 接続・切断コマンド実行
  - [ ] 3.1.2 サーバー一覧取得
  - [ ] 3.1.3 接続状態取得
- [ ] 3.2 認証情報読み込み (設定ファイル流用)
- [ ] 3.3 非同期タスク処理 (tokio)
- [ ] 3.4 エラー処理・ログ出力

---

## Phase 4: Connection Management UI

- [ ] 4.1 サーバーリスト表示
  - [ ] 4.1.1 国別グループ化
  - [ ] 4.1.2 都市・サーバー番号表示
  - [ ] 4.1.3 Ping表示・負荷状況
- [ ] 4.2 サーバー検索 (インクリメンタルサーチ)
- [ ] 4.3 接続・切断ボタン/キー操作
- [ ] 4.4 接続状態インジケーター

---

## Phase 5: Statistics View (btop風)

- [ ] 5.1 リアルタイム速度表示 (DL/UL)
- [ ] 5.2 総通信量表示
- [ ] 5.3 接続時間表示
- [ ] 5.4 サーバー情報表示 (IP, プロトコル, サーバー名)
- [ ] 5.5 ビュー切り替え (統計 ↔ 接続一覧)

---

## Phase 6: Security Features

- [ ] 6.1 Kill Switch機能
  - [ ] 6.1.1 ネットワーク遮断 (iptables/netsh)
  - [ ] 6.1.2 有効/無効切り替えUI
- [ ] 6.2 Secure Core設定
- [ ] 6.3 Always On (自動再接続)
- [ ] 6.4 設定保存・永続化

---

## Phase 7: Vim-style Keybindings

- [ ] 7.1 基本操作 (j/k/h/l 移動)
- [ ] 7.2 検索 (/, n, N)
- [ ] 7.3 終了 (q, ESC)
- [ ] 7.4 ヘルプ表示 (?)
- [ ] 7.5 キーバインド設定画面

---

## Phase 8: Polish & Cross-platform

- [ ] 8.1 テーマ対応 (ライト/ダーク)
- [ ] 8.2 国際化 (i18n) - 日本語対応
- [ ] 8.3 Windows対応
- [ ] 8.4 macOS対応
- [ ] 8.5 パフォーマンス最適化

---

## Phase 9: Distribution

- [ ] 9.1 cargo install対応
- [ ] 9.2 各OSパッケージ作成
- [ ] 9.3 Homebrew対応
- [ ] 9.4 ドキュメント作成

---

## 優先度高 (MVP必須)

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 7
```

## 優先度中 (全部入り所需)

```
Phase 6
```

## 優先度低 (後期改善)

```
Phase 8 → Phase 9
```

---

## 技術メモ

### ディレクトリ構成案

```
src/
├── main.rs           # エントリーポイント
├── app.rs            # メインアプリロジック
├── ui/
│   ├── mod.rs
│   ├── components/   # UIコンポーネント
│   ├── layout/       # レイアウト
│   └── style.rs      # スタイル定義
├── vpn/
│   ├── mod.rs
│   ├── client.rs     # protonvpn-cliラッパー
│   └── state.rs      # 接続状態管理
├── config/
│   ├── mod.rs
│   └── settings.rs   # 設定管理
└── utils/
    ├── mod.rs
    └── error.rs      # エラー定義
```

### 主要依存関係

```toml
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
log = "0.4"
env_logger = "0.10"
anyhow = "1"
thiserror = "1"
```

### キー、バインディング参考

| キー | アクション |
|------|-----------|
| `j` / `↓` | 下一項目 |
| `k` / `↑` | 上一項目 |
| `h` / `←` | パネル戻る |
| `l` / `→` | パネル進む/決定 |
| `/` | 検索開始 |
| `n` | 次検索 |
| `N` | 前検索 |
| `c` | 接続 |
| `d` | 切断 |
| `s` | 統計ビュー |
| `r` | サーバービュー |
| `?` | ヘルプ |
| `q` | 終了 |
| `ESC` | キャンセル |
MJ|| `Tab` | ビュー切り替え |

---

## CI/CD設定詳細

### 提供する価値

| 自動化 |  内容  |  メリット |
|--------|--------|----------|
| **テスト** | `cargo test` を全PRで自動実行 | バグ早期発見、リグレッション防止 |
| **リンティング** | `cargo clippy` で静的解析 | 一般的なミスを自動検出 |
| **フォーマット** | `cargo fmt --check` でコードスタイル統一 | コードレビューが効率的に |
| **ビルド** | 複数OS (Linux/macOS/Windows) でビルド確認 | クロスプラットフォーム対応保証 |
| **ドキュメント** | `cargo doc` でAPIドキュメント自動生成 | 変更時にドキュメント整合性を維持 |

### ワークフロー構成

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build release
        run: cargo build --release
```

### キャッシュ戦略

- **Swatinem/rust-cache@v2**: 依存関係をキャッシュして2回目以降のビルドを高速化
- **ターゲットディレクトリ**: インクリメンタルビルドが可能に

### PRでの効果

```
✅ cargo test     → テスト失敗 → PRをブロック
✅ cargo clippy   → 警告あり → PRをブロック  
✅ cargo fmt      → スタイル違反 → PRをブロック
✅ 複数OSビルド   → プラットフォーム依存のバグを発見
```

### 開発者体験

- **push/Test**: ローカルでテスト漏してもOK → CIが教えてくれる
- **コードレビュー**: スタイルや警告の議論が不要 → 自動チェック
- **.Maintain**: 依存更新時、影響範囲をCIが即座に報告
