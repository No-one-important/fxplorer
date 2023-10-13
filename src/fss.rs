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
}

impl Fst {
    pub fn new(path: String) -> Self {
        let (tx, rx) = mpsc::channel::<String>();

        let mut tree = Fst {
            current_path: path,
            sub_items: vec![],
            search_term: String::new(),
            tx: tx,
            rx: rx,
            stop_tx: None,
        };

        tree.generate_sub_items();

        return tree;
    }

    pub fn generate_sub_items(&mut self) {
        let dir = fs::read_dir(&self.current_path).unwrap();
        self.sub_items = vec![];

        for item in dir {
            self.sub_items
                .push(item.unwrap().path().display().to_string());
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

        match self.stop_tx.clone() {
            Some(s_tx) => {
                s_tx.send(true).unwrap();
                self.stop_tx = None;
            }
            None => {}
        };

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
        self.sub_items = vec![];
        let tx = self.tx.clone();
        let path = self.current_path.clone();
        let search_term = self.search_term.clone();
        let (s_tx, s_rx) = mpsc::channel::<bool>();
        self.stop_tx = Some(s_tx);

        thread::spawn(move || {
            for item in WalkDir::new(path) {
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
                let i: Vec<&str> = item_path.split(MAIN_SEPARATOR).collect();

                if i[i.len() - 1].contains(&search_term) {
                    tx.send(item_path).unwrap();
                }
            }
        });
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
