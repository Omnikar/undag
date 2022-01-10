# Ask the user to type "foo" or "ping", then respond to "foo" with
# "bar", and respond to "ping" with "pong".

git commit --allow-empty -m 'println "Type foo or ping."'
git tag _start
git commit --allow-empty -m 'inpln input'
git commit --allow-empty -m 'branch $input'
git switch -c foo-b
git commit --allow-empty -m 'println "bar"'
git tag foo
git switch main
git switch -c ping-b
git commit --allow-empty -m 'println "pong"'
git tag ping
git switch foo-b
git branch -f main
git switch main
git merge ping-b --no-commit
git commit --allow-empty-message -m ''
git tag _end
git branch -d foo-b ping-b
