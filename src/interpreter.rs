use git2::{Repository, Signature, Error, Oid, Commit, Tree};
use std::collections::HashMap;
use std::borrow::Cow;

use super::command::Command;
use super::command::TreeNode;

const DEFAULT_NAME: &'static str  = "generate-git-repo";
const DEFAULT_EMAIL: &'static str = "generate-git-repo@example.org";

fn print_warning(message: &str) {
    use colored::*;

    let message = format!("WARNING: {}", message);
    println!("{}", message.color("yellow"));
}

#[derive(Debug)]
enum FileToWrite {
    File {
        contents: Vec<u8>
    },
    Directory(HashMap<String, FileToWrite>)
}

fn split_path(path: &str) -> Vec<&str> {
    path.split("/").collect()
}

fn add_to_files_to_write(files_to_write: &mut HashMap<String, FileToWrite>,
                         full_path: &str,
                         path_parts: &[&str],
                         contents: Vec<u8>) {
    
    if path_parts.len() == 1 {
        let name = path_parts[0];

        if files_to_write.contains_key(name) {
            // TODO: should be an error
            panic!("File or directory already exists: {}", full_path);
        }

        // just write the file
        files_to_write.insert(name.to_string(), FileToWrite::File { contents: contents });
    } else if path_parts.len() > 1 {
        let name = path_parts[0];
        let tail = &path_parts[1..];

        // if directory exists, use it
        // otherwise, make it
        if !files_to_write.contains_key(name) {
            files_to_write.insert(name.to_string(), FileToWrite::Directory(HashMap::new()));
        }

        if let Some(directory) = files_to_write.get_mut(name) {
            if let FileToWrite::Directory(hm) = directory {
                add_to_files_to_write(hm, full_path, tail, contents);
            } else {
                // TODO: should be an error
                // TODO: show full directory name
                panic!("Already added as a non-directory: {}", name);
            }
        } else {
            unreachable!();
        }
    } else {
        panic!("Unexpected")
    }
}

fn create_tree_recur(repo: &Repository, tree: &HashMap<String, FileToWrite>) -> Result<Oid, Error> {
    // Git trees are recursive, so it's easy to use a recursive function to make them.

    let mut tree_builder = repo.treebuilder(None)?;

    for (name, node) in tree.iter() {
        match node {
            FileToWrite::File { contents } => {
                let blob_oid = repo.blob(contents)?;
                // File permissions: rw-r--r--
                tree_builder.insert(name, blob_oid, 0o100644)?;
            },
            FileToWrite::Directory(subtree) => {
                let subtree_oid = create_tree_recur(repo, subtree)?;
                // File permissions: directory flag
                tree_builder.insert(name, subtree_oid, 0o040000)?;
            },
        }
    }

    let tree_oid = tree_builder.write()?;
    Ok(tree_oid)
}

fn create_files_to_write(tree: &HashMap<String, TreeNode>) -> HashMap<String, FileToWrite> {
    let mut files_to_write: HashMap<String, FileToWrite> = HashMap::new();

    for (path, node) in tree.iter() {
        // split path by slashes
        let path_parts = split_path(path);

        match node {
            TreeNode::Utf8File(contents) => {
                let contents_vec: Vec<u8> = contents.as_bytes().to_vec();
                add_to_files_to_write(&mut files_to_write, path, &path_parts, contents_vec);
            }
        }
    }

    files_to_write
}

fn create_tree(repo: &Repository, tree: &HashMap<String, TreeNode>) -> Result<Oid, Error> {
    let files_to_write = create_files_to_write(tree);

    // build the tree objects once all the files are known
    create_tree_recur(repo, &files_to_write)
}

pub struct Interpreter<'a> {
    repo: &'a Repository,

    id_to_oid_lookup: HashMap<String, Oid>,

    default_author_name: String,
    default_author_email: String,
    default_committer_name:  String,
    default_committer_email: String,
    default_tagger_name:  String,
    default_tagger_email: String,

    default_tree: Tree<'a>,
}

impl Interpreter<'_> {
    pub fn new(repo: &Repository) -> Result<Interpreter, Error> {
        // Default tree has no files
        let default_tree_oid = create_tree(repo, &HashMap::new())?;
        let default_tree = repo.find_tree(default_tree_oid)?;

        Ok(Interpreter {
            repo,
            id_to_oid_lookup: HashMap::new(),

            default_author_name:  DEFAULT_NAME.to_string(),
            default_author_email: DEFAULT_EMAIL.to_string(),
            default_committer_name:  DEFAULT_NAME.to_string(),
            default_committer_email: DEFAULT_EMAIL.to_string(),
            default_tagger_name:  DEFAULT_NAME.to_string(),
            default_tagger_email: DEFAULT_EMAIL.to_string(),

            default_tree,
        })
    }

    fn get_oid(&self, id: &String) -> Option<Oid> {
        match self.id_to_oid_lookup.get(id) {
            Some(oid) => Some(*oid),
            None => {
                print_warning(&format!("ID doesn't exist: {}", id));
                None
            }
        }
    }

    fn set_oid(&mut self, id: String, oid: Oid) {
        self.id_to_oid_lookup.insert(id, oid);
    }

    pub fn interpret_command(&mut self, command: &Command) -> Result<(), Error> {
        let repo = self.repo;

        match &command {
            Command::Commit { id, message, parents, tree, branches, tags } => {
                // Build the commit's tree
                let tree: Cow<Tree> = if let Some(tree) = tree {
                    // If a tree was provided, build it.
                    // It's a new value that gets freed at the end of this fn, so it's Cow::Owned
                    let tree_oid = create_tree(repo, tree)?;
                    Cow::Owned(repo.find_tree(tree_oid)?)
                } else {
                    // If no tree was provided, use the default tree.
                    // It's an existing ref, so it's Cow::Borrowed
                    Cow::Borrowed(&self.default_tree)
                };
                

                let author    = Signature::now(&self.default_author_name, &self.default_author_email)?;
                let committer = Signature::now(&self.default_committer_name, &self.default_committer_email)?;

                // Resolve { parents: [...] } to git2-rs Commit objects
                let parent_oids: Vec<Oid> = parents.iter().flat_map(|parent_id| {
                    self.get_oid(parent_id)
                }).collect();
                let parent_objects_result: Result<Vec<Commit>, Error> = parent_oids.into_iter().map(|oid| {
                    repo.find_commit(oid)
                }).collect();
                let parent_objects: Vec<Commit> = parent_objects_result?;
                let parent_objects_refs: Vec<&Commit> = parent_objects.iter().collect();

                let used_message: &str = if let Some(ref m) = message {
                    // Use the provided message
                    m
                } else {
                    // Use the commit's ID as the message
                    &id
                };

                // Commit!
                let commit_oid = repo.commit(None, &author, &committer, used_message, &tree, &parent_objects_refs)?;
                self.set_oid(id.to_string(), commit_oid);

                // Create branches
                if let Some(branches) = branches {
                    let commit = repo.find_commit(commit_oid)?;
                    for name in branches {
                        repo.branch(name, &commit, true /* force, even if branch exists */)?;
                    }
                }

                // Create lightweight tags
                if let Some(tags) = tags {
                    let commit = repo.find_object(commit_oid, None)?;
                    for name in tags {
                        repo.tag_lightweight(name, &commit, true /* force, even if tag exists */)?;
                    }
                }
            },
            
            Command::Branch { name, on } => {
                if let Some(commit_oid) = self.get_oid(on) {
                    let commit = repo.find_commit(commit_oid)?;

                    repo.branch(name, &commit, true /* force, even if branch exists */)?;

                    // repo.reference(&format!("refs/remotes/github/{}", name), commit_oid, true, "test")?;
                }
            },
            
            Command::Tag { name, on, lightweight } => {
                if let Some(commit_oid) = self.get_oid(on) {
                    let commit = repo.find_object(commit_oid, None)?;

                    if *lightweight {
                        // Lightweight tag
                        repo.tag_lightweight(name, &commit, true /* force, even if tag exists */)?;
                    } else {
                        // Annotated tag
                        let tagger = Signature::now(&self.default_tagger_name, &self.default_tagger_email)?;

                        repo.tag(name, &commit, &tagger, "Tag message", true /* force, even if tag exists */)?;
                    }
                }
            },

            Command::Config { all_name,       all_email,
                              author_name,    author_email,
                              committer_name, committer_email,
                              tagger_name,    tagger_email,
                              tree } => {
                //
                if let Some(all_name) = all_name {
                    self.default_author_name    = all_name.clone();
                    self.default_committer_name = all_name.clone();
                    self.default_tagger_name    = all_name.clone();
                }
                if let Some(all_email) = all_email {
                    self.default_author_email    = all_email.clone();
                    self.default_committer_email = all_email.clone();
                    self.default_tagger_email    = all_email.clone();
                }
                
                if let Some(author_name) = author_name {
                    self.default_author_name = author_name.clone();
                }
                if let Some(author_email) = author_email {
                    self.default_author_email = author_email.clone();
                }

                if let Some(committer_name) = committer_name {
                    self.default_committer_name = committer_name.clone();
                }
                if let Some(committer_email) = committer_email {
                    self.default_committer_email = committer_email.clone();
                }

                if let Some(tagger_name) = tagger_name {
                    self.default_tagger_name = tagger_name.clone();
                }
                if let Some(tagger_email) = tagger_email {
                    self.default_tagger_email = tagger_email.clone();
                }


                if let Some(tree) = tree {
                    let tree_oid = create_tree(repo, &tree)?;
                    let tree = repo.find_tree(tree_oid)?;
                    self.default_tree = tree;
                }
            },
            
        };

        Ok(())
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_test() {
        assert_eq!(split_path(""),
                   vec![""]);
        
        assert_eq!(split_path("path"),
                   vec!["path"]);
        
        assert_eq!(split_path("path/to/file"),
                   vec!["path", "to", "file"]);
    }
}