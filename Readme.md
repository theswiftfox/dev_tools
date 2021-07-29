# Dev Tools  


## JWT secret  
Either rename `secret.key.sample` to `secret.key` in `src\users`  
or create a new `secret.key` via  
```console
theswiftfox@box:~/dev_tools/src/users$ head -c16 /dev/urandom > secret.key
```

## Cross Compile for pi  
Using [cargo-cross](https://github.com/rust-embedded/cross):  
```console
$ cross build --target armv7-unknown-linux-musleabihf --release
```  
