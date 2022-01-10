# Ask for the user's name, then greet them with `Hello, <name>!`.

git commit --allow-empty -m 'println "What is your name?"'
git tag _start
git commit --allow-empty -m 'inpln name'
git commit --allow-empty -m 'concat message "Hello, " $name'
git commit --allow-empty -m 'concat message $message "!"'
git commit --allow-empty -m 'println $message'
git tag _end
