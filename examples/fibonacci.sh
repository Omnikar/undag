# Print the terms of the Fibonacci sequence until it exceeds 100.

git commit --allow-empty -m 'set a #0'
git tag _start
git commit --allow-empty -m 'println $a'
git commit --allow-empty -m 'set b #1'
git commit --allow-empty -m 'println $b'
git commit --allow-empty -m 'add c $a $b'
git tag loop
git commit --allow-empty -m 'println $c'
git commit --allow-empty -m 'set a $b'
git commit --allow-empty -m 'set b $c'
git commit --allow-empty -m 'gt end $c #100'
git commit --allow-empty -m 'match path $end #0 loop #1 _end'
git commit --allow-empty -m 'branch $path'
git replace --graft loop loop^ HEAD
git commit --allow-empty --allow-empty-message -m ''
git tag _end
