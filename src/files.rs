use std::{fs::DirEntry, path::PathBuf};

use std::fs::read_dir;

pub struct FileSelector {
    running_path : PathBuf,
    contents : Vec<DirEntry>,
}

impl FileSelector {
    pub fn new(start_path : PathBuf) -> Self {
        Self {
            running_path : start_path.clone(),

            contents : Self::read_contents(start_path),
        }
    }
    
    fn read_contents(path : PathBuf) -> Vec<DirEntry> {
        let mut entries: Vec<_> = read_dir(path)
            .unwrap()
            .map(|res| res.unwrap())
            .collect();

        entries.sort_by_key(|dir| dir.path());

        return entries;

    }

    pub fn contents(&self) -> &Vec<DirEntry>{
        return &self.contents;
    }
}