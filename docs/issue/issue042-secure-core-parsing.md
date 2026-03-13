# issue042 - Secure Core 接続の出力対応

## Summary

`protonvpn connect -sc` (Secure Core) の出力形式が異なるため、`parse_connect_output` を修正して `via` フィールドを追加する。

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

## 実装方針

### 変更ファイル

| File | Change |
|------|--------|
| `src/vpn/types.rs` | `ConnectResult` に `via: Option<String>` 追加 + parser 修正 |

### via と Secure Core の関係

- `via.is_some()` → **Secure Core 接続**
- `via.is_none()` → **通常接続**

Secure Core 且つ都市ありの場合のみ `via` が出現。

### ConnectResult (変更後)

```rust
pub struct ConnectResult {
    pub server_id: String,       // 例: "CH-JP#2"
    pub ip: Option<String>,      // 例: "37.19.205.233"
    pub city: Option<String>,    // 例: "Tokyo"
    pub country: Option<String>, // 出口国: "Japan" (Secure Core時はなし)
    pub via: Option<String>,     // エントリ国: "Switzerland" (Secure Core時のみ)
}
```

### Parser ロジック

```rust
// "in Tokyo, via Switzerland" をパース
if let Some(via_idx) = after_server.find(", via ") {
    city = Some(after_server[..via_idx].to_string());
    country = None;
    via = Some(after_server[via_idx + 7..].to_string());
} else if let Some(last_comma_idx) = after_server.rfind(", ") {
    city = Some(after_server[..last_comma_idx].to_string());
    country = Some(after_server[last_comma_idx + 2..].to_string());
    via = None;
} else {
    // Country only (都市なし)
    city = None;
    country = Some(after_server.trim().to_string());
    via = None;
}
```

---

## テストケース追加

```rust
#[test]
fn test_parse_connect_output_secure_core() {
    let output = r#"Connected to CH-JP#2 in Tokyo, via Switzerland.
Your new IP address is 37.19.205.233."#;
    let result = parse_connect_output(output);

    assert_eq!(result.server_id, "CH-JP#2");
    assert_eq!(result.ip, Some("37.19.205.233".to_string()));
    assert_eq!(result.city, Some("Tokyo".to_string()));
    assert_eq!(result.country, None);  // 出口国なし
    assert_eq!(result.via, Some("Switzerland".to_string()));  // エントリ国
}

#[test]
fn test_parse_connect_output_country_only() {
    let output = r#"Connected to JP#374 in Japan.
Your new IP address is 159.26.119.144."#;
    let result = parse_connect_output(output);

    assert_eq!(result.server_id, "JP#374");
    assert_eq!(result.ip, Some("159.26.119.144".to_string()));
    assert_eq!(result.city, None);  // 都市なし
    assert_eq!(result.country, Some("Japan".to_string()));
    assert_eq!(result.via, None);
}
```

---

## 優先度

- **Medium**: 既存の通常接続parserへの影響避免のため、A案を選択
