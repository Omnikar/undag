# Prompt the user to enter items to add to a collection and accept said items.
# When the user enters "done", print what they entered, annotated with indices.

git commit --allow-empty -m 'set items/len #0'
git tag _start
git commit --allow-empty -m 'println "Enter items to add. Enter \"done\" to finish."'
git commit --allow-empty -m 'inpln input'
git tag add-loop
git commit --allow-empty -m 'match path $input done _end $input add-loop'
git commit --allow-empty -m 'branch $path'
git switch -d HEAD
git commit --allow-empty -m 'concat var-name items/ $items/len'
git commit --allow-empty -m 'set $var-name $input'
git commit --allow-empty -m 'concat message "Added \"" $input'
git commit --allow-empty -m 'concat message $message "\""'
git commit --allow-empty -m 'println $message'
git commit --allow-empty -m 'add items/len $items/len #1'
git replace --graft add-loop add-loop^ HEAD
git switch main
git commit --allow-empty -m 'println "You entered:"'
git commit --allow-empty -m 'set i #0'
git commit --allow-empty -m 'match path $i $items/len _end $i print-loop'
git tag print-loop
git commit --allow-empty -m 'branch $path'
git switch -d HEAD
git commit --allow-empty -m 'concat var-name items/ $i'
git commit --allow-empty -m 'get item $var-name'
git commit --allow-empty -m 'concat item ": " $item'
git commit --allow-empty -m 'concat item $i $item'
git commit --allow-empty -m 'println $item'
git commit --allow-empty -m 'add i $i #1'
git replace --graft print-loop print-loop^ HEAD
git switch main
git commit --allow-empty --allow-empty-message -m ''
git tag _end
