use std::{fs::DirEntry, path::PathBuf};

use std::fs::read_dir;

pub struct FileSelector {
    running_path : PathBuf,
    contents : Vec<DirEntry>,

    selected_entry : usize,
    is_file : bool,
}

impl FileSelector {
    pub fn new(start_path : PathBuf) -> Self {
        Self {
            running_path : start_path.clone(),
            contents : Self::read_contents(start_path),
            
            selected_entry : 0,
            is_file : true,
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

    fn eval_selection(&mut self) {
        let entry = self.contents.get(self.selected_entry).unwrap();
        self.is_file = entry.path().is_file();
    } 

    pub fn contents(&self) -> &Vec<DirEntry>{
        return &self.contents;
    }

    pub fn selected_entry(&self) -> usize {
        return self.selected_entry;
    }

    pub fn move_up(&mut self) {
        if self.selected_entry == 0 {
            return;
        }

        self.selected_entry -= 1;
        self.eval_selection();
    }
    pub fn move_down(&mut self) {
        if self.selected_entry >= self.contents().len() - 1 {
            return;
        }

        self.selected_entry += 1;
        self.eval_selection();
    }
    pub fn move_back(&mut self) {
            self.running_path.pop();
            self.selected_entry = 0;
            self.contents = FileSelector::read_contents(self.running_path.clone());
            self.eval_selection();
    }
    pub fn move_forwards(&mut self) {
        if !self.is_file {
            self.running_path = self.contents().get(self.selected_entry).unwrap().path();
            self.selected_entry = 0;
            self.contents = FileSelector::read_contents(self.running_path.clone());
            self.eval_selection();
        }
    }
    pub fn queue_selection(&self) -> Option<Vec<DirEntry>> {
        if !self.is_file {
            let path = self.contents().get(self.selected_entry).unwrap().path().clone();
            return Some(Self::read_contents(path));
        }

        return None;
    }
}