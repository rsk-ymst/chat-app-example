### デバッグ実行手順
#### 0. 接続設定
examples/cli.rs の対象アドレスを設定する

#### 1. サーバを起動させる
```
cargo run
```

#### 2. クライアントを起動
別ターミナルウィンドウを用意し、

```
cargo run --example cli
```
- Successfully connected と表示された段階で、無限ループの標準入力待機状態になる
- 各クライアントには、ランダムに生成した32文字のIDと、5文字のnameが付与される


### 対応済コマンド

#### 部屋作成
```
/create
```
- 送信者がオーナーとなるルームを作成する
- 新しい部屋IDが返ってくる

#### 入室
```
/join <room_id>
```
- 部屋にはいる

#### ACK
```
/ack
```
- 進行の可否を承認する

#### ACKキャンセル
```
/rm_ack
```
- 進行の可否の承認をキャンセルする
- `ack_cancel` にしようかと思ったけど、タイポを防ぐために短くした


### デバッグ用コマンド
おそらくデバッグ時にしか使わないコマンド

```
/list
```
- 現在ある部屋とその作成者、部屋にいるユーザの一覧を送信
- 以下のフォーマットになる
```
received: Text("865afdd4-855a-4360-81b5-5bf39dbe7b19 by l3jr3")
received: Text("1: rYt647VckVrDHMqbRFX5ypYzaTmt91Xd, l3jr3")
received: Text("656e7472-795f-5f5f-5f5f-5f5f5f5f5f5f by admin")
received: Text("1: jm6hQVho9AHHZgjj6J2KTDFEsVrew4Or, Aq61e")
received: Text("2: WSx3uSs8eCEoJ1DzyvFRwn03wpDTDKKV, N6yeI")
```
上記の場合、サーバには
- l3jr3がつくった部屋, 865afdd4-855a-4360-81b5-5bf39dbe7b19
- adminが作った部屋, 656e7472-795f-5f5f-5f5f-5f5f5f5f5f5f

が存在し、各部屋にはそれぞれ`l3jr3`, `Aq61e`と`N6yeI`がいることを示す。

### ログ見方
サーバサイドのログ出力は、基本的に以下のフォーマットに従う

```
[テキストメッセージの種類] <送信したクライアントの名前>: <補足情報>
```

通常のテキストメッセージを受信した場合、
```
[MESSAGE] john: <テキスト内容>
```

ACKコマンドを受信した場合、
```
[ACK] john: <ACK受信に成功したかどうかのメッセージ文>
```

というように表示される。
