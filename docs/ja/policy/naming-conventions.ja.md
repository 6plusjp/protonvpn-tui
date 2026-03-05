# 命名規則

## ファイルとディレクトリ

- ファイル名すべて: 小文字、ハイフン区切り（kebab-case）
- イシュードキュメント: `issue001.md`, `issue002.md`（3桁ゼロ埋め）
- 日本語版: `.md` の前に `.ja` を追加
  - 英語版: `requirements.md` → 日本語版: `requirements.ja.md`
- ファイル名やディレクトリ名にスペースを含めない

## コード（言語に依存しないデフォルト）

- 変数と関数: camelCase
- 定数: UPPER_SNAKE_CASE
- クラスと型: PascalCase
- プライベートメンバー: アンダースコア `_` をプレフィックス

## ブランチ名

- 機能開発: `feat/issue<NNN>-<short-description>`
  - 例: `feat/issue003-user-authentication`
- バグ修正: `fix/issue<NNN>-<short-description>`
- ドキュメント: `docs/issue<NNN>-<short-description>`

## 注意

- 言語固有の規則はこれらのデフォルトより優先される
- プロジェクトに応じて言語固有の規則をこのファイルに追加すること
