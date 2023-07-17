// file system support

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

//file system tree
pub struct Fst {
    pub current_path: String,
    pub sub_items: Vec<String>
}

impl Fst {
    fn new(path: String) -> Self {
        let mut tree = Fst {
            current_path: path,
            sub_items: vec![],
        };

        tree.generate_sub_items();

        return tree;
    }

    fn generate_sub_items(&mut self) {
        let dir = fs::read_dir(&self.current_path).unwrap();

        for item in dir {
            self.sub_items.push(item.unwrap().path().display().to_string());
        }
    }


    fn change_dir(&mut self, path: String) {
        let mut path_buf = PathBuf::from(&self.current_path);

        path_buf.push(path);

        self.current_path = path_buf.to_string_lossy().to_string();
    }

    // TODO: add regex support
    fn search(&self, search_term: String) -> Vec<String> {
        let mut results = vec![];

        for item in WalkDir::new(&self.current_path) {
            let item_path: String = item.unwrap().path().display().to_string();

            if item_path.contains(&search_term) {
                results.push(item_path);
            }
        }

        results

    }
}
