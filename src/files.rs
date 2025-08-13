use std::{fs::DirEntry, path::PathBuf};

use std::fs::{read_dir, ReadDir};

use color_eyre::eyre::Error;

use crate::expand_tilde;

pub struct FileSelector {
    running_path : PathBuf,
    contents : Vec<DirEntry>,

    selected_entry : usize,
    is_file : bool,
}

impl FileSelector {
    pub fn new(start_path : PathBuf) -> Self {
        let mut path = start_path.clone();

        let contents = 
        match Self::read_contents(start_path.clone()) {
            Ok(res) => res,
            _ => {
                path = expand_tilde("~/");
                Self::read_contents(path.clone()).unwrap()
            }
        };

        Self {
            running_path : start_path.clone(),
            contents : contents,
            
            selected_entry : 0,
            is_file : true,
        }
    }
    
    fn read_contents(path : PathBuf) -> Result<Vec<DirEntry>, Error> {
        let mut entries: Vec<_> = read_dir(path)?
            .map(|res| res.unwrap())
            .collect();

        entries.sort_by_key(|dir| dir.path());

        return Ok(entries);

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
            self.contents = FileSelector::read_contents(self.running_path.clone()).unwrap();
            self.eval_selection();
    }
    pub fn move_forwards(&mut self) {
        if !self.is_file {
            self.running_path = self.contents().get(self.selected_entry).unwrap().path();
            self.selected_entry = 0;
            self.contents = FileSelector::read_contents(self.running_path.clone()).unwrap();
            self.eval_selection();
        }
    }
    pub fn queue_selection(&self) -> Option<Vec<DirEntry>> {
        if !self.is_file {
            let path = self.contents().get(self.selected_entry).unwrap().path().clone();
            return Some(Self::read_contents(path).unwrap());
        }

        return None;
    }
}