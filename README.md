# suprps

Press the **PS button** on your controller to launch your preferred game launcher.  
This tool, created for personal use, simplifies startup inside a virtual machine for those who prefer an ultra-minimal interface.  
Edit the configuration file to suit your setup.

### Build a release

```sh
cargo build --release
cp ./target/release/suprps /your/desired/path
```

### Example with launch in Hyprland
Add this line to your Hyprland config: `exec-once = suprps`

**Make sure `suprps` is in your PATH.**

## See log
journalctl -e -t suprps -f

Todo

- native disconnection from device
