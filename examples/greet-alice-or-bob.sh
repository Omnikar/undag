# Ask for the user's name, then, if their name is "Alice" or "Bob", greet them
# with `Hello, <name>!`. Otherwise, refuse to greet them.

git commit --allow-empty -m 'println "What is your name?"'
git tag _start
git commit --allow-empty -m 'inpln name'
git commit --allow-empty -m 'match path $name Alice greet Bob greet $name reject'
git commit --allow-empty -m 'branch $path'
git switch -c greet-b
git commit --allow-empty -m 'concat message "Hello, " $name'
git tag greet
git commit --allow-empty -m 'concat message $message "!"'
git switch main
git switch -c reject-b
git commit --allow-empty -m 'set message "Sorry, I only greet Alice and Bob."'
git tag reject
git switch greet-b
git branch -f main
git switch main
git merge reject-b -m 'println $message'
git tag _end
git branch -d greet-b reject-b
