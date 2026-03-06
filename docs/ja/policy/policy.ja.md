# プロジェクトポリシー

## ドキュメント

- すべてのドキュメントは `docs/` に配置し、情報源とする
- `docs/` のコンテンツ（`docs/ja/` を除く）は英語 で記述
- `docs/ja/` には `.ja.md` サフィックスを付けた日本語訳を配置
- `docs/reference/` はユーザーが管理するもの — 決して作成・編集しない

## ワークフロー

- すべてのタスクは `docs/issue/` のイシュードキュメントから始める
- イシューファイルは連番: issue001.md, issue002.md, ...
- イシュードキュメントが存在しない状態で実装を開始しない
- コード変更と同じコミットでドキュメントも更新する
- イシューが解決したら、ファイル名に `resolved_` プレフィックスを付ける（例: `issue002.md` → `resolved_issue002.md`）

## ポリシーの更新

- ポリシーファイルの変更は事前にユーザーと相談する
- ポリシー変更には英語版と日本語版の両方更新が必要

## 関連ポリシーファイル

- [@docs/policy/commit-message-rule.md](docs/policy/commit-message-rule.md) — コミットメッセージの形式
- [@docs/policy/naming-conventions.md](docs/policy/naming-conventions.md) — ファイル、コード、ブランチの命名規則
- [@docs/policy/reference-convention.md](docs/policy/reference-convention.md) — ドキュメント参照の慣例
