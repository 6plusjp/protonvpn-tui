# Codebase Improvements

## 優先順位表

| 優先度 | 項目                            | 状態   | 作業量 |
| ------ | ------------------------------- | ------ | ------ |
| ✅     | 設定カウント計算の重複除去      | 完了   | 小     |
| ✅     | ConnectionState 重複の解決      | 完了   | 中     |
| ✅     | 選択メソッドの統一的改善        | 完了   | 小     |
| ✅     | 非同期処理の tokio 化           | 完了   | 大     |
| ✅     | connect_random の非同期化       | 完了   | 小     |
| ✅     | ファイルの分割                  | 完了   | 中     |
| ✅     | AsyncTaskManager のメモリリーク | 完了   | 中     |
| ✅     | Clippy warning の修正           | 完了   | 小     |
| ✅     | Settings 呼び出しのキャッシュ   | 完了   | 小     |
| ✅     | notification_log の上限設定     | 完了   | 小     |
| ✅     | EnterAlternateScreen 重複       | 完了   | 極小   |
| ✅     | マジック Numbers の定数化       | 完了   | 小     |
| **低** | テストの追加                    | 未着手 | 中     |
| **低** | VpnState API の対称性修正       | 未着手 | 小     |
| **低** | generate_fuzzy_variants 最適化  | 未着手 | 小     |
| **低** | help_view のキー更新           | 未着手 | 小     |
| **低** | AppState の分割 (God Object)    | 未着手 | 大     |

---

## 未完了の問題点

### 低優先度

#### #1: テストの不在

`tests/` ディレクトリなし。

#### #2: VpnState API の非対称性

`get_servers(&mut self)`、`list_servers(&mut self)` は `&self` で十分。

#### #3: generate_fuzzy_variants の HashMap

毎回 `HashMap::from([...])` を作成している。`lazy_static` を使用。

#### #4: help_view のキー不一致

- 不足: `x` (random connect), `g/G`, `Ctrl+d/u`
- 不要な説明が含まれている可能性

#### #5: AppState の巨大化 (God Object)

単一責任原則の違反。DomainState / UiState / OperationState に分割。
