// file system support

use std::fs;
use std::path::MAIN_SEPARATOR;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use walkdir::WalkDir;

//file system tree
pub struct Fst {
    pub current_path: String,
    pub sub_items: Vec<String>,
    pub search_term: String,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
    pub stop_tx: Option<Sender<bool>>,
    pub show_hidden_files: bool,
    pub searching: bool, // show full relative file paths when searching
}

impl Fst {
    pub fn new(path: String, show_hidden_files: bool) -> Self {
        let (tx, rx) = mpsc::channel::<String>();

        let mut tree = Fst {
            current_path: path,
            sub_items: vec![],
            search_term: String::new(),
            tx: tx,
            rx: rx,
            stop_tx: None,
            show_hidden_files: show_hidden_files,
            searching: false,
        };

        tree.generate_sub_items();

        return tree;
    }

    pub fn generate_sub_items(&mut self) {
        let dir = fs::read_dir(&self.current_path).unwrap();
        self.sub_items = vec![];

        for item in dir {
            let path = item.unwrap().path().display().to_string();

            if self.show_hidden_files {
                self.sub_items.push(path);
            } else {
                if !is_hidden(&path) {
                    self.sub_items.push(path);
                }
            }
        }
    }

    pub fn change_dir(&mut self, path: &str) {
        if path == ".." {
            let cp = self.current_path.clone();
            let cps: Vec<&str> = cp.split(MAIN_SEPARATOR).collect();
            let mut path = String::new();

            for i in 0..(cps.len() - 1) {
                path = path + cps[i] + &MAIN_SEPARATOR.to_string();
            }
            self.current_path = path;
        } else {
            self.current_path = path.to_string();
        }

        // remove slashes from end of path
        if self.current_path.ends_with(MAIN_SEPARATOR) {
            self.current_path = self.current_path.remove_last().to_string();
        }

        // add slash if windows disk letter
        if self.current_path.ends_with(":") {
            self.current_path = self.current_path.clone() + &MAIN_SEPARATOR.to_string();
        }

        self.generate_sub_items();
    }

    // open if file change dir if folder
    pub fn action(&mut self, path: &str) {
        self.stop_search();

        let md = fs::metadata(&path).unwrap();
        if md.is_dir() || path == ".." {
            self.change_dir(path);
        } else {
            //open file
            open::that(path).ok();
        }
    }

    // TODO: add regex support
    pub fn search(&mut self) {
        //stop other searches
        self.stop_search();

        self.searching = true;

        self.sub_items = vec![];
        let tx = self.tx.clone();
        let path = self.current_path.clone();
        let search_term = self.search_term.clone();
        let (s_tx, s_rx) = mpsc::channel::<bool>();
        self.stop_tx = Some(s_tx);

        let show_hidden_files = self.show_hidden_files;
        thread::spawn(move || {
            // dont search in hidden dirs if not showing hidden files
            let walker = WalkDir::new(path).into_iter().filter_entry(|e| !is_hidden(e.path().to_str().unwrap_or("")) || show_hidden_files);

            for item in walker {
                let mut stop: bool = false;
                match s_rx.try_recv() {
                    Ok(x) => {
                        stop = x;
                    }
                    Err(_) => {}
                };

                if stop {
                    break;
                }

                let item_path: String = item.unwrap().path().display().to_string();

                if !show_hidden_files {
                    if is_hidden(&item_path) {
                        continue;
                    }
                }

                let i: Vec<&str> = item_path.split(MAIN_SEPARATOR).collect();

                if i[i.len() - 1].contains(&search_term) {
                    tx.send(item_path).unwrap();
                }
            }
            
//            self.searching = false;
        });
    }

    fn stop_search(self: &mut Self) {
        if self.searching {
            self.searching = false;

            self.stop_tx.clone().unwrap().send(true).ok(); // fails if search finished
            self.stop_tx = None;
        }
    }
}

trait StrExt {
    fn remove_last(&self) -> &str;
}

impl StrExt for str {
    fn remove_last(&self) -> &str {
        match self.char_indices().next_back() {
            Some((i, _)) => &self[..i],
            None => self,
        }
    }
}

#[cfg(windows)]
use std::os::windows::prelude::*;

#[cfg(windows)]
fn is_hidden(file_path: &str) -> bool {
    let metadata = match fs::metadata(file_path) {
        Ok(x) => x,
        Err(_) => {
            println!("error getting metadata for: {file_path}");
            return false;
        }, // dont show error files
    };
    let attributes = metadata.file_attributes();

    if (attributes & 0x2) > 0 {
        true
    } else {
        false
    }
}

#[cfg(unix)]
fn is_hidden(file_path: &str) -> bool {
    let i: Vec<&str> = item_path.split(MAIN_SEPARATOR).collect();

    i[i.len() - 1].starts_with('.')
}
