use serde::{Deserialize};
use std::collections::HashMap;

// Used as serde deserialization defaults
fn empty_vec_string() -> Vec<String> { vec![] }
fn false_boolean() -> bool { false }

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TreeNode {
  Utf8File(String),
  
  // possible feature: accept an object with arguments, such as file permissions
}


#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "commit")]
    // #[serde(other)] 
    Commit {
        id: String,

        message: Option<String>,

        #[serde(default = "empty_vec_string")]
        parents: Vec<String>,

        tree: Option<HashMap<String, TreeNode>>,

        // If these are set, assign branches/tags to the commit
        branches: Option<Vec<String>>,
        tags:     Option<Vec<String>>
    },

    #[serde(rename = "merge")]
    Merge {
        id: String,

        commits: Vec<String>,

        // Only used if a merge commit is made
        message: Option<String>,
        tree: Option<HashMap<String, TreeNode>>,

        // If these are set, assign branches/tags to the commit
        branches: Option<Vec<String>>,
        tags:     Option<Vec<String>>,

        // Disable fast-forward merges. Fast-forward is enabled by default.
        #[serde(default = "false_boolean")]
        no_ff: bool,
    },
    
    #[serde(rename = "branch")]
    Branch {
        name: String,
        on: String,
    },

    
    #[serde(rename = "tag")]
    Tag {
        name: String,
        on: String,

        #[serde(default = "false_boolean")]
        lightweight: bool
    },

    #[serde(rename = "config")]
    Config {
      all_name:       Option<String>,   all_email:       Option<String>,
      author_name:    Option<String>,   author_email:    Option<String>,
      committer_name: Option<String>,   committer_email: Option<String>,
      tagger_name:    Option<String>,   tagger_email:    Option<String>,

      tree: Option<HashMap<String, TreeNode>>,
    }
}