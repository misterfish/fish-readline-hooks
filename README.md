# fish-readline-hooks
An embedded language for readline transformations in `bash`.

### What does this do?

This is a working implementation of a little language that will add more wow to your command line.

### Examples

In these examples ```<ctl-k>``` means literally press ctl-k. You can choose your own magic key of course.

Basically, everything from ```=``` through to the ```<ctl-k>``` gets run through the mini-interpreter and replaced.

```bash

# --- move the last file (by timestamp) from the ~/Downloads directory here.
mv -iv = ~/Downloads <ctl-k> .

# --- view the last pdf (by alphanum sort) in the "~/book shelf" directory.
view-pdf = ~/bookshelf l <ctl-k>

# --- cat the 3rd-to-last file (by timestamp) in /tmp.
cat = /tmp 3 t <ctl-k>
```

The syntax is: `=` `dir` `num` `command`

`command`:
- `t`: last file (optionally offset by `num` by timestamp)
- `l`: last file (optionally offset by `num` by alphanumeric sort)
- `tr`: like `t` but reversed.
- `lr`: like `l` but reversed.
- `z`: undo previous command: if you don't like the replacement you got, add `= z<ctl-K>` and it will revert.
  This one doesn't take a dir or a num.

In most unambiguous cases `dir` and `num` are optional (default to `.` and `1` respectively). `command` defaults to `t`.

I'll probably be expanding the language as the need arises. Or you can bug me if there's something you want.

### Installation.

- git clone https://github.com/misterfish/fish-readline-hooks
- cd fish-readline-hooks
- git submodule update --init --recursive
- ./build

In your .bashrc:

```bash
fish-readline-hooks() {
    local path="<path to the project>/target/release/readline-hooks"
    binder=$(
        printf '%s%s%s' '"\C-k":eval $(' "$path" ')'
    )
    bind -x "$binder"
}; fish-readline-hooks
```

Start a new shell and you should be good to go.


