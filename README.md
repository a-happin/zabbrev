# zabbrev

[![](https://github.com/a-happin/zabbrev/actions/workflows/build.yml/badge.svg)](https://github.com/a-happin/zabbrev/actions/workflows/build.yml)
<!-- [![](https://badgen.net/crates/v/zabrze)](https://crates.io/crates/zabrze) -->

ZSH abbreviation expansion plugin

## Feature

- No compatible with [original repository](https://github.com/Ryooooooga/zabrze)

## Usage

### Configuration

#### (root)

- **`abbrevs` is required**

|key|value type|
|:-:|:-:|
|abbrevs|List\<Abbr\>|

#### Abbr

- **required any one of `abbr`, `abbr-prefix`, `abbr-suffix` or `abbr-regex`**
- **required any one of `replace-self`, `replace-first`, `replace-context`, `replace-all`, `append` or `prepend`**

|Category|key|value type|description|
|:-:|:-:|:-:|---|
||name|Option\<String\>|abbreviation name|
||context|String|default is `""` (empty)<br>only trigger if the command string (including arguments) starts with `context`|
||global|bool|● `false`: disallow extra arguments. (default)<br>● `true`: allow extra arguments.|
|Trigger|abbr|String|trigger if the last argument is `abbr`|
|^|abbr-prefix|String|trigger if the last argument starts with `abbr-prefix`|
|^|abbr-suffix|String|trigger if the last argument ends with `abbr-suffix`|
|^|abbr-regex|String|trigger if the last argument matches `abbr-regex`|
|Operation|replace-self|String|replace the last argument with `snippet`|
|^|replace-first|String|replace the first argument with `snippet`|
|^|replace-context|String|replace the matched context with `snippet`|
|^|replace-all|String|replace whole command with `snippet`|
|^|append|String|insert `snippet` after the last argument|
|^|prepend|String|insert `snippet` before the first argument|
||cursor|Option\<String\>|placeholder|
||evaluate|bool|● `false`: insert as string (default)<br>● `true`: do zsh parameter expansion, then insert|
||redraw|bool|● `false`: do nothing (default)<br>● `true`: force to reset prompt after expansion<br>(Note: set `true` if there is a problem with the display)|

### Special variables

Following variables are available if `evaluate == true`

|name|description|
|:-:|-|
|`$1` , `$2`, ... |expands to arguments that removed matched context)|

### Setup

In your `.zshrc`

```zsh
eval "$(zabbrev init --bind-keys)"
```

### Examples

### Simple abbreviation

behaves like zsh aliases

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # simple abbreviation
  - name: git
    abbr: 'g'
    replace-self: 'git'

  - name: editor
    abbr: 'e'
    replace-self: '${EDITOR}'
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
    append: '-i'
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
    prepend: 'sudo'
```

then

```zsh
$ apt<Space>
#  ↓ expanded
$ sudo apt 
```

### Subcommand abbreviation

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # subcommand abbreviation
  - name: git commit
    context: 'git'
    abbr: 'c'
    replace-self: 'commit'

  - name: git push -u origin HEAD
    context: 'git'
    abbr: 'pu'
    replace-self: 'push -u origin HEAD'

  # subcommand abbreviation with evaluate
  - name: git pull --rebase origin CURRENT_BRANCH
    context: 'git'
    abbr: 'pr'
    replace-self: 'pull --rebase origin $(git symbolic-ref --short HEAD)'
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
    abbr-suffix: '.tar'
    replace-first: 'tar -xvf'

  - name: compress tar
    context: 'compress'
    abbr-suffix: '.tar'
    replace-first: 'tar -cvf'
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
  - name: run jar file
    abbr-suffix: '.jar'
    prepend: 'java -jar'
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
    abbr-prefix: ''
    replace-all: 'mkdir -p $1 && cd $1'
    evaluate: true
```

then

```zsh
$ mkdircd foo<Space>
#  ↓ expanded
$ mkdir -p foo && cd foo 
```

### Global abbreviation

behaves like zsh global abbreviations

```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # global abbreviation
  - name: '>/dev/null'
    abbr: 'null'
    replace-self: '>/dev/null'
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

  # fix option
  - name: replace -f with --force-with-lease
    context: 'git push'
    abbr: '-f'
    replace-self: '--force-with-lease'
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

$ git push -f<Space>
#  ↓ expanded
$ git push --force-with-lease
```

### As one pleases


```yaml
# ~/.config/zsh/zabbrev.yaml
abbrevs:
  # as one pleases
  - context: 'cd'
    abbr: 'f'
    replace-self: $(fd --type d --hidden --no-ignore --exclude .git | fzf --preview 'exa -lha --time-style long-iso --color=always {}')
    evaluate: true
    redraw: true

  - context: 'cd'
    abbr: 'g'
    replace-self: $(fd --type d --hidden --follow '^.git$' ~ -x dirname | fzf --preview 'git -c color.status=always -C {} status')
    evaluate: true
    redraw: true

  # choose commit interactively
  - context: 'git rebase'
    abbr: '-i'
    append: $(git log --graph --all --oneline --color=always | fzf --ansi --no-sort --reverse --tiebreak index -0 --height=60% --preview "git show --color=always \$(printf '%s' {} | grep -io '[0-9a-f]\{7,\}' | head -1)" | \grep -io '[0-9a-f]\{7,\}' | head -1)
    evaluate: true
    redraw: true

  # ?????
  - context: 'rm -i'
    abbr-prefix: ''
    replace-context: rm $([[ -d $1 ]] && printf '%s' '-ri' || printf '%s' '-i')
    evaluate: true

  # [[  ]]
  - abbr: '[['
    replace-self: '[[ 🐣 ]]'
    cursor: '🐣'
```

then

```zsh
$ cd f<Space>
#  ↓ expanded
$ cd ./Downloads

$ git i<Space>
#  ↓ expanded
$ git rebase -i 544f368

$ [[<Space>
#  ↓ expanded
$ [[ | ]]
# cursor is at '|'
```

## Installation

```sh
$ git clone https://github.com/a-happin/zabbrev.git && cd zabbrev && cargo install --path .
```

## Alternatives
- [zabrze](https://github.com/Ryooooooga/zabrze) (original repository)
- [zsh-abbrev-alias](https://github.com/momo-lab/zsh-abbrev-alias)
- [zeno.zsh](https://github.com/yuki-yano/zeno.zsh)
