#!/usr/bin/env node

// This script generates commands to be consumed by generate-git-repo.

// Run with:
// node ./create-tree.js | generate-git-repo --bare ./tree-repo


/**** Some helper functions ****/
const commands = []
function addCommand(command) { commands.push(command) }

let currentId = 0
function makeId() {
  currentId += 1
  return `${currentId}`
}


/**** The real work ****/
function createTree(n, numberOfChildren, parent=null, parentsSoFar=[]) {
  const parents = parent ? [parent] : []

  const id = makeId()
  const path = [...parentsSoFar, id]

  addCommand({ type: "commit",
               id: id,
               message: `Commit ${id}`,
               parents: parents,
               tree: { "path.txt": path.join(" -> ") } })

  if (n <= 1) {
    // Commit is leaf node. Add a tag for each leaf node.
    addCommand({ type: "tag", name: `tag-${id}`, on: id })
  } else {
    for (let i = 0; i < numberOfChildren; i++) {
      createTree(n-1, numberOfChildren, id, path)
    }
  }
}

// Generate the commands, print as JSON to stdout
createTree(3, 2)
console.log(JSON.stringify(commands, null, 2))




/*

n=1:

  o


n=2; numberOfChildren=2:

  o o
  |/
  o


n=3; numberOfChildren=2:

  o o   o o
   \|   |/
    o   o
     \ /
      o


n=2; numberOfChildren=3

  o o o
   \|/
    o

*/