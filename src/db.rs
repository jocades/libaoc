struct Database {
    path: PathBuf,
}

/// A pair of year and day.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Id(u32, u32);

impl Default for Id {
    fn default() -> Self {
        Self(2024, 1)
    }
}

impl Id {
    fn to_path(&self) -> PathBuf {
        self.into()
    }

    #[inline]
    fn year(&self) -> u32 {
        self.0
    }

    #[inline]
    fn day(&self) -> u32 {
        self.1
    }
}

#[derive(Default)]
struct Bucket {
    id: Id,
    path: PathBuf,
}

impl Puzzle {
    pub fn read(id: Id) -> Self {
        todo!()
    }
}

impl From<(u32, u32)> for Id {
    fn from((y, d): (u32, u32)) -> Self {
        Self(y, d)
    }
}

impl From<&Id> for PathBuf {
    fn from(Id(y, d): &Id) -> Self {
        Path::new(&y.to_string()).join(&d.to_string())
    }
}

impl Bucket {
    pub fn write(&self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
        fs::write(self.path.join(path), data)
    }
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().into(),
        }
    }

    pub fn get(&self, id: impl Into<Id>) -> Option<Puzzle> {
        let id: Id = id.into();
        assert!(2015 <= id.year() && id.year() < 2025);
        assert!(1 <= id.day() && id.day() < 25);
        let path = self.path.join("puzzle.json");
        path.exists()
            .then(|| serde_json::from_reader(File::open(&path).unwrap()).unwrap())
    }

    pub fn bucket_path(&self, id: &Id) -> PathBuf {
        self.path.join(id.to_path())
    }
}

/* match db.get((year, day)) {
    Some(bucket) => bucket,
    None => {
        let mut url = aoc_url(year, day);
        let html = client.get(&url).send().context("get view")?.text()?;
        let doc = Html::parse_document(&html);
        let selector = Selector::parse("article.day-desc").unwrap();
        fs::create_dir_all(&dir)?;
        let mut view = File::create(dir.join("view"))?;
        for (i, article) in doc.select(&selector).enumerate() {
            if i > 1 {
                eprintln!("found more than 2 articles");
                break;
            }
            let text = html2text::from_read(article.inner_html().as_bytes(), 80)?;
            view.write_all(text.as_bytes())?;
            // fs::write(dir.join())
        }
        url.push_str("/input");
        let mut input = File::create(dir.join("input"))?;
        let mut resp = client.get(&url).send().context("get input")?;
        io::copy(&mut resp, &mut input).context("write input")?;
        db.get((year, day));
        todo!()
    }
}; */
