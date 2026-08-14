# timeowt

一個簡單的時間排程小工具。

- 一次性：等到指定時間執行腳本
- 每天重複：自動寫入 crontab

## 下載

[點這裡下載 Linux 版本](https://github.com/yangycl/timeowt/releases/latest/download/timeowt)

## 使用方式

```bash
# 一次性
timeowt 14 30 ./backup.sh

# 每天固定時間
timeowt -re 14 30 /home/user/backup.sh
```
