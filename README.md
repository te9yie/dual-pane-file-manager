# dual-pane-file-manager

Rustの勉強がてら作るTUI2画面ファイラ

srcウィンドウとdestウィンドウの2画面
srcウィンドウが操作対象

## Usage

- `q` 終了する
- `j` カーソルを下に移動する
- `k` カーソルを上に移動する
- `h` 親ディレクトリに移動する
- `l` 子ディレクトリに移動する
- `g` カーソルを一番上に移動する
- `G` カーソルを一番下に移動する
- `/` ディレクトリ内の名前検索
- `Tab` srcウィンドウを切り替える
- ` ` マークを付ける/外す
- `Enter` 実行する (*)
- `e` 編集する (*)

*: 設定ファイルで指定する。

これから
- `c` コピー
- `m` 移動
- `d` 削除
- `o` srcディレクトリをdestディレクトリと同じにする
- `i` ディレクトリ作成

## Customize

`~/.config/dual-pane-file-manager/settings.json` が設定ファイル。
実行・編集に何を使うかを設定する。`%p` がファイルパスに変換される。

```
{
    "exec_command": {
        "program": "explorer",
        "args": "%p"
    },
    "edit_command": {
        "program": "gvim",
        "args": "%p"
    }
}
```

CUIなやつは `tmux new-window vim %p` とかにしておけばいいんじゃないのかな…
