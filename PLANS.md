# Proton VPN TUI

## Goal

`protonvpn-app`をTUIで代替する。

### 現在の課題

- 現在の接続先が見えない
- 検索ができない
- vimのように操作できない

## Tech Stack

- **Language**: Rust
- **TUI Library**: crossterm (軽量・高速・シンプルなAPI)
- **VPN接続**: protonvpn-cli (既存認証情報を流用)
- **アーキテクチャ**: 単一バイナリ (daemon化は将来検討)

## Features

### 接続管理

- VPN接続/切断
- サーバー選択 (国 > 都市 > サーバー番号)
- 接続状態表示

### 統計表示 (btop風ビュー切り替え)

- リアルタイムダウンロード/アップロード速度
- セッション中の総通信量
- 接続時間
- サーバー情報 (IP, プロトコル, サーバー名)

### セキュリティ機能

- Kill Switch (VPN切断時にネットワーク遮断)
- Secure Core (経由サーバー强制)
- Always On (自動再接続)

### UI/UX

- vim風キーバインド (j/k/h/l, /検索, q終了など)
- ビュー切り替え可能 (接続一覧 / 統計 / 設定)
- インクリメンタルサーチ対応

## Target OS

- Linux
- macOS
- Windows

## Distribution

- パッケージマネージャー対応 (cargo install, 各OSのパッケージ)

## References

- btop (UI reference)
- protonvpn-cli (backend)

## Phase

- **MVP**: 1-2週間で動作する基盤作成
- **将来**: daemon + client分離対応可能性
