# cpx

An enhanced cp command written in Rust, including a progress bar.

## Examples

quiet copy (default)
```
cpx file1 file2 ~/temp
```

progress copy
```
cpx --progress big.tar.gz my/sshfs/mount
```

Looks like:
```
copying bigfile to bigfilecopy
[694.91 MiB/s] [ETA 10s] #############--------------------------- 3.03 GiB of 10.00 GiB 30% Complete   
```
