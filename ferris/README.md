

# Rusn


[Async/Tokio](docs/assets/async.png)

[theadpool](docs/assets/threadpool.png)

[rayon](docs/assets/rayon.png)


# Run all

`./target/debug/ferris --compare /home/jesse/Documents`

# Record specific

`psrecord "./target/debug/ferris --implementation rayon /home/jesse/Documents" --log rayon.log --interval 0.25`

# Record plot 

`psrecord "./target/debug/ferris --implementation threadpool /home/jesse/Documents" --interval 0.5  --plot threadpool.png`

# Testing

TEst directory is N levels random deep & files are 256kb - 4096MB. Contains are palintext 

```shell
├── 2dbehhyh54
│   ├── 3358v1hslp5k.log
│   ├── c0811digsaaw.txt
│   ├── d686o5ngfgfa.log
│   ├── dtjhsbpwsj
│   │   ├── 7992lywnw1mx.txt
│   │   ├── c2kuzfjrgm21.cfg
│   │   ├── c5qyxk9zr8
│   │   ├── ebsj2s32jt
│   │   ├── h0hbktt2c72j.log
...

215046 directories, 536981 files
```
