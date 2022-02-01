# zabbrev

[![](https://github.com/a-happin/zabbrev/actions/workflows/build.yml/badge.svg)](https://github.com/a-happin/zabbrev/actions/workflows/build.yml)
<!-- [![](https://badgen.net/crates/v/zabrze)](https://crates.io/crates/zabrze) -->

ZSH abbreviation expansion plugin

## Feature

Differences from [original repository](https://github.com/Ryooooooga/zabrze)
- Fewer to construct regex
- Advanced operation
- No compatible with [original repository](https://github.com/Ryooooooga/zabrze)

## Usage

### Configuration

|(Root)|key|value type|
|---|:-:|:-:|
||abbrevs|List\<Abbr\>|

|Abbr|key|value type|description|
|---|:-:|:-:|---|
||name|Option\<String\>|abbreviation name|
||context|String|default is `""` (empty)<br>[see below](#Customize-conditions)|
||global|bool|default is `false`<br>[see below](#Customize-conditions)|
||abbr|String|a trigger string **(required either `abbr` or `abbr-regex`)**|
|^|abbr-regex|String|a trigger regex **(required either `abbr` or `abbr-regex`)**|
||snippet|String|the string to be expanded **(required)**|
||operation|String|expansion method<br>● `replace-self`: replace the last argument with `snippet` (default)<br>● `replace-command`: replace the first argument with `snippet`<br>● `replace-all`: replace whole command with `snnipet`<br>● `append`: insert `snnipet` after the last argument<br>● `prepend`: insert `snippet` before the first argument|
||evaluate|bool|● `false`: insert as string (default)<br>● `true`: do zsh parameter expansion, then insert|

### Customize conditions

| | `context == ""` | `context != ""` |
|:-:|:-:|:-:|
|`global == false`|only trigger at the first argument|only trigger at the second argument|
|`global == true`|trigger anywhere |trigger anywhere if the first argument is `context`|

### Special variables

Following variables are available if `evaluate == true`

|name|description|
|:-:|-|
|`$1`| expands to trigger string|

### Setup

In your `.zshrc`

```zsh
$ eval "$(zabbrev init --bind-keys)"
```

### Examples

### Normal abbreviations

behaves like zsh aliases

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # normal abbreviations
  - name: git
    abbr: 'g'
    snippet: 'git'

  - name: editor
    abbr: 'e'
    snippet: '${EDITOR}'
    evaluate: true
```

then

```zsh
$ g<Space>
#  ↓ expanded
$ git 

$ e<Space>
#  ↓ expanded
$ nvim 
```

### Add default option

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # add default option
  - name: mv -i
    abbr: 'mv'
    snippet: '-i'
    operation: append
```

then

```zsh
$ mv<Space>
#  ↓ expanded
$ mv -i 
```
### Prepend sudo

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # prepend sudo
  - name: sudo apt
    abbr: 'apt'
    snippet: 'sudo'
    operation: prepend
```

then

```zsh
$ apt<Space>
#  ↓ expanded
$ sudo apt 
```

### Subcommand abbreviations

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # subcommand abbreviations
  - name: git commit
    context: 'git'
    abbr: 'c'
    snippet: 'commit'

  - name: git push -u origin HEAD
    context: 'git'
    abbr: 'pu'
    snippet: 'push -u origin HEAD'

  - name: git pull --rebase origin CURRENT_BRANCH
    context: 'git'
    abbr: 'pr'
    snippet: 'pull --rebase origin $(git symbolic-ref --short HEAD)'
    evaluate: true
```

then

```zsh
$ git c<Space>
#  ↓ expanded
$ git commit 

$ git pu<Enter>
#  ↓ expanded
$ git push -u origin HEAD

$ git pr<Enter>
#  ↓ expanded
$ git pull --rebase origin main
```

### Fake command

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # fake command
  - name: extract tar
    context: 'extract'
    abbr-regex: '\.tar$'
    snippet: 'tar -xvf'
    operation: replace-command

  - name: compress tar
    context: 'compress'
    abbr-regex: '\.tar$'
    snippet: 'tar -cvf'
    operation: replace-command
```

then

```zsh
$ extract archive.tar<Enter>
#  ↓ expanded
$ tar -xvf archive.tar

$ compress archive.tar<Space>
#  ↓ expanded
$ tar -cvf archive.tar 
```

### Associated command

behaves like zsh suffix aliases

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # associated command
  - name: run java file
    abbr-regex: '\.java$'
    snippet: 'java -jar'
    operation: prepend
```

then

```zsh
$ ./main.jar<Space>
#  ↓ expanded
$ java -jar ./main.jar 
```

### Like a function

behaves like zsh functions

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # like a function
  - name: mkdircd
    context: 'mkdircd'
    abbr-regex: '.+'
    snippet: 'mkdir -p $1 && cd $1'
    operation: replace-all
    evaluate: true
```

then

```zsh
$ mkdircd foo<Space>
#  ↓ expanded
$ mkdir -p foo && cd foo 
```

### Global abbreviations

behaves like zsh global abbreviations

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # global abbreviations
  - name: >/dev/null
  - abbr: 'null'
    snippet: '>/dev/null'
    global: true
```

then

```zsh
$ type cargo null<Space>
#  ↓ expanded
$ type cargo >/dev/null 
```

### Global abbreviations with context

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # global abbreviations with context
  - name: git current branch
    context: 'git'
    abbr: 'B'
    snippet: '$(git symbolic-ref --short HEAD)'
    global: true
    evaluate: true
```

then

```zsh
$ git show B<Space>
#  ↓ expanded
$ git show main 

$ echo B<Space>
#  ↓
$ echo B 
```

### As one pleases


```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # as one pleases
  # You don't have to remember shortcut key.
  - context: 'cd'
    abbr: 'f'
    snippet: $(fd --type d --hidden --no-ignore --exclude .git | fzf --preview 'exa -lha --time-style long-iso --color=always {}')
    evaluate: true
  - context: 'cd'
    abbr: 'g'
    snippet: $(fd --type d --hidden --follow '^.git$' ~ -x dirname | fzf --preview 'git -c color.status=always -C {} status')
    evaluate: true
  # choose commit interactively
  - context: 'git'
    abbr: 'i'
    snippet: rebase -i $(git log --graph --all --oneline --color=always | fzf --ansi --no-sort --reverse --tiebreak index -0 --height=60% --preview "git show --color=always $(echo -- \"{}\" | grep -io '[0-9a-f]\{7,\}' | head -1)" | grep -io '[0-9a-f]\{7,\}' | head -1)
    evaluate: true
```

then

```zsh
$ cd f<Space>
#  ↓ expanded
$ cd ./Downloads

$ git i<Space>
#  ↓ expanded
$ git rebase -i 544f368
```

## Installation

```sh
$ git clone https://github.com/a-happin/zabbrev.git && cd zabbrev && cargo install --path .
```

## Alternatives
- [zabrze](https://github.com/Ryooooooga/zabrze) (original repository)
- [zsh-abbrev-alias](https://github.com/momo-lab/zsh-abbrev-alias)
- [zeno.zsh](https://github.com/yuki-yano/zeno.zsh)
