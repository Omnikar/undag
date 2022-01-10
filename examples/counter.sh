# Use a loop to print the numbers from 0 to 10.

git commit --allow-empty -m 'set count #0'
git tag _start
git commit --allow-empty -m 'println $count'
git tag counter-loop
git commit --allow-empty -m 'add count $count #1'
git commit --allow-empty -m 'gt end $count #10'
git commit --allow-empty -m 'match path $end #0 counter-loop #1 _end'
git commit --allow-empty -m 'branch $path'
git replace --graft counter-loop counter-loop^ HEAD
git commit --allow-empty --allow-empty-message -m ''
git tag _end
