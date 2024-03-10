use log::{debug, error, info};
use std::{collections::HashMap, env, path::Path};

/// The Router struct
/// 
/// The Router struct is responsible for initializing and storing the routes of the server
/// 
/// # Fields
/// 
/// * `routes` - A HashMap of the routes
pub struct Router {
    routes: HashMap<String, String>,
}

impl Router {
    /// Create a new Router
    ///
    /// The Router will initialize the routes from the pages directory
    ///
    /// # Returns
    ///
    /// A new Router instance
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the current directory cannot be found
    /// or if the pages directory cannot be found
    ///
    pub fn new() -> Self {
        let routes = Self::init_routes();
        Router { routes }
    }

    /// Get the routes
    pub fn get_routes(&self) -> &HashMap<String, String> {
        &self.routes
    }

    /// Initialize the routes
    ///
    /// The routes are initialized by reading the pages directory
    ///
    fn init_routes() -> HashMap<String, String> {
        debug!("Initializing routes...");
        let current_dir = env::current_dir().expect("Failed to get current directory");
        let root_dir = current_dir.join("pages");

        let mut routes = HashMap::new();
        Self::read_path(&root_dir, &mut routes);

        info!("Routes: {:#?}", routes);
        routes
    }

    fn read_path(dir: &Path, map: &mut HashMap<String, String>) {
        let root_path = dir.strip_prefix(env::current_dir().unwrap()).unwrap();
        let root_page = remove_first_occurrence(root_path.to_str().unwrap(), "pages");

        for entry in dir.read_dir().expect("Failed to read directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();
            if path.is_dir() {
                // Recursively read the directory
                Self::read_path(&path, map);
            } else {
                // Add the file to the map
                match path.strip_prefix(env::current_dir().unwrap()) {
                    Ok(stripped_path) => {
                        match stripped_path.to_str() {
                            Some(p) => {
                                if p.contains("index.html") {
                                    // If the file is index.html, add it to the root page
                                    debug!("page: {:#?}, path: {:#?}", root_page, p);
                                    map.insert(
                                        String::from(root_page.to_string()),
                                        String::from(p.to_string()),
                                    );
                                } else {
                                    // Otherwise, add it to the map
                                    let page = if p.contains("html") {
                                        path.strip_prefix(env::current_dir().unwrap())
                                            .unwrap()
                                            .with_extension("")
                                    } else {
                                        path.strip_prefix(env::current_dir().unwrap())
                                            .unwrap()
                                            .to_path_buf()
                                    };
                                    let page_str =
                                        remove_first_occurrence(page.to_str().unwrap(), "pages");
                                    debug!("page: {:#?}, path: {:#?}", page_str, p);
                                    map.insert(page_str, String::from(p.to_string()));
                                }
                            }
                            None => {
                                error!("Failed to convert path to string");
                            }
                        };
                    }
                    Err(_) => {
                        error!("Failed to strip prefix from path: {:#?}", path);
                    }
                }
            }
        }
    }
}

/// Remove the first occurrence of a pattern from a string
///
/// # Arguments
///
/// * `input` - The input string
/// * `pattern` - The pattern to remove
///
fn remove_first_occurrence(input: &str, pattern: &str) -> String {
    if let Some(index) = input.find(pattern) {
        let (_, rest) = input.split_at(index);
        let result_str = rest.trim_start_matches(pattern);

        if result_str.is_empty() {
            "/".to_string()
        } else {
            result_str.to_string()
        }
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_logger::SimpleLogger;

    #[test]
    fn test_init_routes() {
        SimpleLogger::new().init().unwrap();
        info!("Testing init_routes");

        let router = Router::new();
        assert!(router.get_routes().len() > 0);
    }

    #[test]
    fn test_remove_first_occurrence() {
        let input = "pages/index.html";
        let pattern = "pages";
        let result = remove_first_occurrence(input, pattern);
        assert_eq!(result, "/index.html");
    }
}
