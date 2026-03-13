# issue042 - Secure Core 接続の出力対応

## Summary

`protonvpn connect -sc` (Secure Core) の出力形式が異なるため、`parse_connect_output` を修正して `via` フィールドを追加する。

**Status**: ✅ Resolved

---

## 背景

### 通常接続

```
Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144.
```

### Secure Core 接続

```
Connected to CH-JP#2 in Tokyo, via Switzerland. Your new IP address is 37.19.205.233.
```

差分:
- `CH-JP#2`: `CH` = エントリ国 (瑞士), `JP` = 出口国 (日本)
- `in Tokyo, via Switzerland`: 都市名の後に `via {entry_country}` が続く

---

## 出力フォーマット (公式CLI Sourceより)

| タイプ | 出力 | server_id | city | country | via |
|--------|------|-----------|------|---------|-----|
| Secure Core + City | `Connected to CH-JP#2 in Tokyo, via Switzerland.` | CH-JP#2 | Tokyo | None | Switzerland |
| Normal + City | `Connected to JP#374 in Tokyo, Japan.` | JP#374 | Tokyo | Japan | None |
| Normal only | `Connected to JP#374 in Japan.` | JP#374 | None | Japan | None |

IPアドレスはオプション。

---

## 実装内容

### 変更ファイル

| File | Change |
|------|--------|
| `src/vpn/types.rs` | `ConnectResult` に `via: Option<String>` 追加 + parser 修正 |
| `src/vpn/cache.rs` | `ServerCache` に `connected_via` 追加、`set_connected()` 拡張 |
| `src/state/connection_state.rs` | `ConnectionState::Connected` に `via` 追加 |
| `src/state/connection.rs` | `AsyncEvent::Connected` と `AsyncEvent::ConnectCityResult` に `via` 追加 |
| `src/state/app_state.rs` | すべての `ConnectionState::Connected` 生成箇所を更新 |
| `src/vpn/client.rs` | `set_connected()` 呼び出しを更新 |
| `src/ui/app.rs` | header 表示ロジックを更新 (Secure Core 場合 `via` を表示) |
| `tests/state_test.rs` | テストの更新 |

### via と Secure Core の関係

- `via.is_some()` → **Secure Core 接続**
- `via.is_none()` → **通常接続**

Secure Core 且つ都市ありの場合のみ `via` が出現。

### ConnectResult

```rust
pub struct ConnectResult {
    pub server_id: String,       // 例: "CH-JP#2"
    pub ip: Option<String>,      // 例: "37.19.205.233"
    pub city: Option<String>,    // 例: "Tokyo"
    pub country: Option<String>,  // 出口国: "Japan" (Secure Core時はなし)
    pub via: Option<String>,     // エントリ国: "Switzerland" (Secure Core時のみ)
}
```

### Header 表示

 Secure Core 接続の場合:
```
CH-JP#2 ip:37.19.205.233 loc:Tokyo,Japan via Switzerland
```

 通常接続の場合:
```
JP#374 ip:159.26.119.144 loc:Tokyo,Japan
```

### Parser ロジック

```rust
if let Some(via_idx) = after_server.find(", via ") {
    city = Some(after_server[..via_idx].to_string());
    via = Some(after_server[via_idx + 6..period_idx].to_string());
    country = None;
} else if let Some(period_idx) = after_server.find('.') {
    let location_part = &after_server[..period_idx];
    if let Some(last_comma_idx) = location_part.rfind(", ") {
        city = Some(location_part[..last_comma_idx].to_string());
        country = Some(location_part[last_comma_idx + 2..].to_string());
    } else {
        city = None;
        country = Some(location_part.to_string());
    }
}
```

---

## テスト

追加したテスト:
- `test_parse_connect_output_secure_core` - Secure Core + IP
- `test_parse_connect_output_secure_core_no_ip` - Secure Core + IPなし
- `test_parse_connect_output_country_only` - 国のみ

全テスト OK (145テストパス)。

---

## 優先度

- **High**: 既存のSecure Core接続のパースが機能しない問題の修正
