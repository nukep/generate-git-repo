use serde::{Deserialize};
use std::collections::HashMap;

// Used as serde deserialization defaults
fn empty_string() -> String { "".to_string() }
fn empty_vec_string() -> Vec<String> { vec![] }
fn false_boolean() -> bool { false }

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TreeNode {
  File(String),
  Tree(HashMap<String, TreeNode>)
}


#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "commit")]
    // #[serde(other)] 
    Commit {
        id: String,

        #[serde(default = "empty_string")]
        message: String,

        #[serde(default = "empty_vec_string")]
        parents: Vec<String>,

        tree: Option<HashMap<String, TreeNode>>,

        // If these are set, assign branches/tags to the commit
        branches: Option<Vec<String>>,
        tags:     Option<Vec<String>>
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