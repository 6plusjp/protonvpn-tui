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
- ✅ サーバーリスト表示 (国別)
- ✅ Fuzzy検索
- ✅ vim風キーバインド (j/k, gg/G, Ctrl+d/u)
- ✅ ヘッダー表示 (接続状態 + IP)
- ✅ サーバーキャッシュ (起動時読込)
- ✅ 'r'でリスト更新

### Additional Implemented Features

- ✅ Countries/Cities階層ナビゲーション (issue002)
  - 国選択 → Enter → 都市リスト表示
  - 都市ごとのFeatures表示 (P2P, Secure Core等)
- ✅ 設定画面 (issue001)
  - Enterキーで設定トグル (protonvpn config set)
  - SpaceキーでDNS無効化
  - Custom DNS入力プロンプト
- ✅ Kill Switch設定トグル (protonvpn config set)
- ✅ IPv6, Moderate NAT, VPN Accelerator, Port Forwarding設定
- ✅ 統計ビュー (btop風) (issue006)
- ✅ ログビュー (通知履歴) (issue006)
- ✅ テーマ切り替え (ダーク/ライト) (issue007)
- ✅ UI Componentsレイヤー作成 (issue007)
  - unified block/list styles
  - 選択項目の `> ` prefix
  - 接続中を示す `* ` prefix
- ✅ キャッシュ整合性改善 (issue004)
  - Arc<VpnState>でスレッドセーフ共有
- ✅ Citiesキャッシュ問題修正 (issue003, issue005)
- ✅ Settingsナビゲーション改善 (issue010)

---

### Remaining Features

#### Phase 4: Connection Management UI

- [ ] 4.1 サーバーリスト表示 (改良)
  - [x] 4.1.1 国別グループ化
  - [x] 4.1.2 都市・サーバー番号表示
  - [ ] 4.1.3 Ping表示・負荷状況 (protonvpn-cliが提供しないため未実装)
- [x] 4.2 統計ビュー (btop風)
  - [ ] 4.2.1 リアルタイム更新 (Rx/Txを/systemから毎秒取得)
- [x] 4.3 設定画面

#### Phase 5: Security Features

- [x] 5.1 Kill Switch機能
- [ ] 5.2 Secure Core設定 (protonvpn-cliで未サポート)
- [ ] 5.3 Always On (protonvpn-cliで未サポート)

#### Phase 6: Polish

- [x] 6.1 テーマ対応 (ライト/ダーク)
- [ ] 6.2 国際化 (i18n)

---

## Issue Tracking

### Resolved Issues

| Issue    | Title                                            | Status      |
| -------- | ------------------------------------------------ | ----------- |
| issue001 | Settings view - Enter key triggers config toggle | ✅ Resolved |
| issue002 | Servers view - Country list with city details    | ✅ Resolved |
| issue003 | Cities view bugs - navigation, cache, parsing    | ✅ Resolved |
| issue004 | Cache Inconsistency - Async Operations Sync      | ✅ Resolved |
| issue005 | Server view - Cities not loaded from cache       | ✅ Resolved |
| issue006 | Add Logs view with notification logs             | ✅ Resolved |
| issue007 | Create UI Components Layer and Unify Design      | ✅ Resolved |
| issue010 | Settings Navigation Refactor                     | ✅ Resolved |

---

## Notes

- **Linux only**: protonvpn-cliがLinuxのみで動作するため、macOS/Windows対応は計画しない
- **Ping/負荷状況**: `protonvpn-cli`がサーバー負荷情報を提供しないため実装不可
- **リアルタイム統計**: `/sys/class/net/tun0/statistics/rx_bytes`, `tx_bytes` から取得可能 (Linuxのみ)
