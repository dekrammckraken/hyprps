# ascps

Press the **PS button** on your controller to launch your preferred game launcher.  
This tool, created for personal use, simplifies startup inside a virtual machine for those who prefer an ultra-minimal interface.  
Edit the configuration file to suit your setup.

---

### Part of the [Cereal setup](https://github.com/dekrammckraken/cereal)

---

### Build a release

```sh
cargo build --release
cp ./target/release/hyprps /your/desired/path
```

### Run in Hyprland
Add this line to your Hyprland config: `exec-once = ascps`

**Make sure `ascps` is in your PATH.**

## See log
journalctl -e -t ascps -f
