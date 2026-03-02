- 基本`protonvpn`コマンドはバックグラウンドで動作させ、tuiには影響させない

## boot

- `protonvpn status`コマンドが存在しないため、vpnに接続しているかどうかは`proton0`で判断
- 初回起動時はvpn接続の有無しかわからないため、接続対象は`unknown`と表示
- `protonvpn countries`をバックグラウンドで走らせている間、cacheでサーバーリストを表示

## connect

- `protonvpn connect`コマンドを使用

```sh
$ protonvpn connect
Connected to JP#378 in Tokyo, Japan. Your new IP address is 159.26.119.143.
```

- どこに接続できているかは`proton0`では判断できないので、返り値を待ち*Connected*から始まる1文があるかチェックする
- この際`server_id`,`city`,`country`,`ip`を保存
- `server_id`はの`ip`

## disconnect

- `protonvpn disconnect`コマンドを使用
- `disconnect`コマンド自体なにも返さないので、`proton0`で判断

## refresh

- `protonvpn countries`コマンドを使用
- 接続先がわかっている場合はhighlightかつ一番上に表示
- 完了後、現在のソートに合わせて並び替え
