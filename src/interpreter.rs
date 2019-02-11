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

fn create_tree(repo: &Repository, tree: &HashMap<String, TreeNode>) -> Result<Oid, Error> {
    let mut tree_builder = repo.treebuilder(None)?;

    for (name, node) in tree.iter() {
        match node {
            TreeNode::File(contents) => {
                let blob_oid = repo.blob(contents.as_bytes())?;
                // File permissions: rw-r--r--
                tree_builder.insert(name, blob_oid, 0o100644)?;
            },

            TreeNode::Tree(subtree) => {
                let subtree_oid = create_tree(repo, subtree)?;
                // File permissions: directory flag
                tree_builder.insert(name, subtree_oid, 0o040000)?;
            }
        }
    }

    let tree_oid = tree_builder.write()?;

    Ok(tree_oid)
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


                // Commit!
                let commit_oid = repo.commit(None, &author, &committer, message, &tree, &parent_objects_refs)?;
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
