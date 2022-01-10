# UnDAG
UnDAG is an esoteric programming language where the program is a
Git repository — the repository does not track the program, but rather is
itself the program. Program instructions are written in the commit messages
of the repository. Execution starts at the commit tagged `_start`, and
moves forward in the commit history until the commit tagged `_end`.
Control flow is created using branches and tags.

## Usage
Build.
```bash
cargo build --release
```
Run a repository as a program.
```bash
undag <repo>
```

## Introduction

### Syntax
As mentioned, program instructions are written in commit messages. Each commit
contains one instruction. Instruction invocations are formatted similarly to
shell commands, with the instruction name followed by a space and then
space-delimited arguments, and one can include a space as part of an argument
by escaping it with a backslash (`foo\ bar`) or quoting the whole argument
(`"foo bar"`). This shell-like behavior means that text is interpreted as
strings by default, regardless of whether its quoted. This includes numerical
arguments. The value of a variable can be used as an argument by prefixing
the variable name with `$` (`$foo`). Note, however, that unlike with shell
commands, variable access via `$` cannot be performed within a string
argument to interpolate the value of the variable into the string
(INVALID: `"foo $bar baz"`); variable accesses must be standalone arguments.
As mentioned, even numerical arguments are interpreted as strings by default;
however, variables can be set to numerical values by prefixing the number
with `#` (`#16`). Currently, only integers are supported.

### Hello World
This will print `Hello, world!` and a newline.

Create a commit whose message is `println "Hello, world!"`.
```bash
git commit --allow-empty -m 'println "Hello, world!"'
```
Since this is the only commit in the program, it is the start and the end
commit, so give it the `_start` and `_end` tags.
```bash
git tag _start
git tag _end
```

### Greeting by Name
This will ask for the user's name, then greet them with `Hello, <name>!`.

Ask the user for their name.
```bash
git commit --allow-empty -m 'println "What is your name?"'
```
Tag the first commit with `_start`
```bash
git tag _start
```
Read a line from stdin using `inpln` and store it in the `name` variable.
```bash
git commit --allow-empty -m 'inpln name'
```
Use `concat` to join `Hello, ` and the inputted name, and store the result in
the `message` variable.
```bash
git commit --allow-empty -m 'concat message "Hello, " $name'
```
Use `concat` to join `!` to the end of the current value of `message`, and
store the result back in the `message` variable.
```bash
git commit --allow-empty -m 'concat message $message "!"'
```
Lastly, print the value stored in the `message` variable.
```bash
git commit --allow-empty -m 'println $message'
```
Tag the end commit with `_end`.
```bash
git tag _end
```

## Branching
Control flow can be created using Git branches, tags, and the `branch`
instruction. The `branch` instruction takes the name of a tag as an
argument, and when it is invoked, it will direct execution down the
path that has the shortest distance from the tag, thus moving
"towards" the commit with that tag.

### Foo Bar, Ping Pong
This will ask the user to type "foo" or "ping", then respond to "foo" with
"bar", and respond to "ping" with "pong".

Ask the user to type "foo" or "ping", then read a line from stdin and store
it in the `input` variable. Tag the first commit with `_start`.
```bash
git commit --allow-empty -m 'println "Type foo or ping."'
git tag _start
git commit --allow-empty -m 'inpln input'
```
Use the `branch` instruction, passing in the value of `input`.
```bash
git commit --allow-empty -m 'branch $input'
```
Now, execution will attempt to move in the direction of the tag
whose name was inputted by the user.

Switch to a new branch for the `foo` case. Here, the name `foo-b` is being
used to prevent ambiguity with the future `foo` tag.
```bash
git switch -c foo-b
```
Print `bar`.
```bash
git commit --allow-empty -m 'println "bar"'
```
Tag the commit with the `foo` tag.
```bash
git tag foo
```
Switch back to the default branch (assumed here to be `main`).
```bash
git switch main
```
Switch to a new branch for the `ping` case.
```bash
git switch -c ping-b
```
Print `pong`.
```bash
git commit --allow-empty -m 'println "pong"'
```
Tag the commit with the `ping` tag.
```bash
git tag ping
```
Now, the two branches must converge to the ending commit. Switch back to the
`foo` case branch, move the `main` branch there, and switch to it.
```bash
git switch foo-b
git branch -f main
git switch main
```
At this point, since there are no more steps that the program needs to
complete before finishing, the ending commit message should be empty.
Since `git merge` does not allow the `--allow-empty-message` flag,
the `--no-commit` flag must be used, followed by a commit using the
`--allow-empty-messaage` flag.
```bash
git merge ping-b --no-commit
git commit --allow-empty-message -m ''
```
Tag the end commit with `_end`.
```bash
git tag _end
```
At this point, `foo-b` and `ping-b` can be deleted, but do not delete
the `foo` and `ping` tags.
```bash
git branch -d foo-b ping-b
```

## Looping
Since execution moves forward through Git history, it may seem unclear how
looping may occur. Looping is, in fact, achieved through the use of
[`git replace`](https://git-scm.com/docs/git-replace) with the `--graft` flag
to (somewhat) create cyclic commit histories, since `git replace --graft`
allows specifying a commit to have any arbitrary list of parents. Of course,
branching must be used to specify whether a loop exits or continues looping.

### Counter
This will use a loop to print the numbers from 0 to 10.

Use `set` to create a counter starting at 0. Note the usage of `#` to make
it an integer value. Tag the starting commit with `_start`.
```bash
git commit --allow-empty -m 'set count #0'
git tag _start
```
Print the value of `count`. This will be the first commit in the loop, so give
it a tag, such as `counter-loop`.
```bash
git commit --allow-empty -m 'println $count'
git tag counter-loop
```
Add 1 to the value of `count`, and store the result back in `count`.
```bash
git commit --allow-empty -m 'add count $count #1'
```
Use `gt` to check if `count` is greater than 10, setting the `end` variable
to 1 if it is, and 0 otherwise.
```bash
git commit --allow-empty -m 'gt end $count #10'
```
We now need to use the 0-or-1 value of `end` to choose a tag to send execution
towards. For this, we can use `match`. Use `match` to set `path` to
`counter-loop` if `end` is 0, and `_end` if `end` is 1.
```bash
git commit --allow-empty -m 'match path $end #0 counter-loop #1 _end'
```
Use `branch` to send execution in the direction of the chosen tag.
```bash
git commit --allow-empty -m 'branch $path'
```
Now, in order to allow execution to move from this commit back to the start
of the loop, we need to add this commit as a parent of the commit at the
start of the loop by using `git replace --graft`.
```bash
git replace --graft counter-loop counter-loop^ HEAD
```
Finally, we need to add an empty-message ending commit.
```bash
git commit --allow-empty --allow-empty-message -m ''
git tag _end
```

## Tables
Currently undocumented.

## Examples
More example programs (without explanations) can be found in the
[examples directory](examples/) in the form of shell scripts containing Git
commands which will generate the program repository.

## Instruction List
| Invocation | Description |
| ---------- | ----------- |
| `set <var> <src>` | Set the variable named `var` to the value given by `src`. |
| `get <var> <src>` | Set the variable named `var` to the value of the variable named `src`. |
| `del <var>` | Delete the variable named `var`. |
| `exists <var> <symbol>` | Set the variable named `var` to 1 if a variable named `symbol` exists, and 0 otherwise. |
| `branch <tag>` | Send execution in the direction of the shortest path to the commit tagged with `tag`. |
| `enter <table>` | Change current table to `table`. |
| `exit` | Change current table to parent of current table. |
| `match <var> <src> [<branch> <val>]...` | Find the first value of `branch` equal to the value given by `src`, then set `var` to the corresponding `val`. |
| `print <arg>` | Print the value given by `arg` to stdout, without a trailing newline. |
| `println <arg>` | Print the value given by `arg` to stdout, with a trailing newling. |
| `inpln <var>` | Read a line from stdin, trimming the trailing newline, and store the result in `var`. |
| `concat <var> <a> <b>` | Concatenate the string representations of `a` and `b`, storing the result in `var`. |
| `chars <var> <string>` | Separate `string` into characters, create a table with variables that store the characters whose names correspond to the character indices and a `len` variable with the number of characters, and store the result in `var`.
| `eq <var> <a> <b>` | Set `var` to 1 if `a` and `b` are equal, and 0 otherwise. |
| `gt <var> <a> <b>` | Set `var` to 1 if `a` is greater than `b`, and 0 otherwise. |
| `add <var> <a> <b>` | Add `a` and `b`, storing the result in `var`. |
| `sub <var> <a> <b>` | Subtract `b` from `a`, storing the result in `var`. |
| `mul <var> <a> <b>` | Multiply `a` and `b`, storing the result in `var`. |
| `div <var> <a> <b>` | Divide `a` by `b`, storing the result in `var`. |
| `mod <var> <a> <b>` | Perform a modulo on `a` and `b`, storing the result in `var`. |
| `and <var> <a> <b>` | Perform a bitwise "and" on `a` and `b`, storing the result in `var`. |
| `or <var> <a> <b>` | Perform a bitwise "or" on `a` and `b`, storing the result in `var`. |
| `xor <var> <a> <b>` | Perform a bitwise "xor" on `a` and `b`, storing the result in `var`. |

## What does UnDAG mean?
Git histories are [**D**irected **A**cyclic **G**raphs](https://en.wikipedia.org/wiki/Directed_acyclic_graph),
which have no cycles. This property, however, is bypassable through the usage
of `git replace --graft`, which is what this esoteric programming language
does, hence "Un-DAG".
