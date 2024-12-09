use std::{fs, io, path::Path};

pub type PuzzleId = (u32, u32);

#[derive(Debug, Default)]
pub struct Puzzle {
    pub id: PuzzleId,
    pub q1: String,
    pub q2: String,
    pub a1: String,
    pub a2: String,
}

impl Puzzle {
    pub fn read(path: impl AsRef<Path>, id: PuzzleId) -> Option<Puzzle> {
        let path = path.as_ref();
        path.exists().then(|| Puzzle {
            id,
            q1: fs::read_to_string(path.join("question1")).unwrap_or_default(),
            q2: fs::read_to_string(path.join("question2")).unwrap_or_default(),
            a1: fs::read_to_string(path.join("answer1")).unwrap_or_default(),
            a2: fs::read_to_string(path.join("answer2")).unwrap_or_default(),
        })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        fs::write(path.join("question1"), self.q1.as_bytes())?;
        fs::write(path.join("question2"), self.q2.as_bytes())?;
        fs::write(path.join("answer1"), self.a1.as_bytes())?;
        fs::write(path.join("answer2"), self.a2.as_bytes())?;
        Ok(())
    }
}
