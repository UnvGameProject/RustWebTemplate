use std::{
    env,
    io::{self, ErrorKind},
    path::PathBuf,
};

#[derive(Debug)]
pub(crate) struct Project {
    root: PathBuf,
}

impl Project {
    pub(crate) fn discover() -> io::Result<Self> {
        if let Ok(root) = env::var("RUSTY_PROJECT_ROOT") {
            let root = PathBuf::from(root);

            if Self::looks_like_project_root(&root) {
                return Ok(Self { root });
            }

            return Err(io::Error::new(
                ErrorKind::NotFound,
                "RUSTY_PROJECT_ROOT does not point to a valid project root",
            ));
        }

        let mut candidate = env::current_dir()?;

        loop {
            if Self::looks_like_project_root(&candidate) {
                return Ok(Self { root: candidate });
            }

            if !candidate.pop() {
                break;
            }
        }

        Err(io::Error::new(
            ErrorKind::NotFound,
            "could not locate the project root",
        ))
    }

    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub(crate) fn migrations_dir(&self) -> PathBuf {
        self.root.join("migrations")
    }

    fn looks_like_project_root(path: &std::path::Path) -> bool {
        if !path.join("Cargo.toml").is_file() {
            return false;
        }

        path.join("compose.yaml").is_file()
            || path.join("compose.yml").is_file()
            || path.join("docker-compose.yaml").is_file()
            || path.join("docker-compose.yml").is_file()
    }
}
