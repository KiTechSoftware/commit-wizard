# cw completions

Print a shell completion script to standard output. It works anywhere, with no repository.

```sh
cw completions bash > ~/.local/share/bash-completion/completions/cw
cw completions zsh > ~/.zfunc/_cw
cw completions fish > ~/.config/fish/completions/cw.fish
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.
