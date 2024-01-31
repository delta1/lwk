use std::{
    fmt::Display,
    fs::{self},
    ops::Add,
    path::{Path, PathBuf},
    str::FromStr,
};

use elements::bitcoin::hashes::{sha256, Hash};

use crate::{ElementsNetwork, Error, Update, WolletDescriptor};

pub trait Persister {
    /// Return the elements in the same order as they have been inserted
    fn iter(&self) -> Box<dyn ExactSizeIterator<Item = Result<Update, Error>> + '_>; // TODO return impl ExactSizeIterator<Item = Update> once MSRV reach 1.75

    /// Push and persist an update. Returns the number of updates persisted
    ///
    /// Implementors are encouraged to coalesce consequent updates with `update.only_tip() == true`
    fn push(&mut self, update: Update) -> Result<usize, Error>;
}

pub struct NoPersist {}

impl NoPersist {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Persister for NoPersist {
    fn iter(&self) -> Box<dyn ExactSizeIterator<Item = Result<Update, Error>>> {
        Box::new([].into_iter())
    }

    fn push(&mut self, _update: Update) -> Result<usize, Error> {
        Ok(0)
    }
}

pub struct FsPersister {
    /// Directory where the data files will be written
    path: PathBuf,

    /// Next free position to write an update
    next: Counter,
}
impl FsPersister {
    /// Creates a persister of updates, writing files in `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Box<Self>, Error> {
        let path = path.as_ref().to_path_buf();
        if path.is_file() {
            return Err(Error::Generic("given path is a file".to_string()));
        }
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        let mut next = Counter::default();
        for el in path.read_dir()? {
            let entry = &el?;
            if entry.path().is_file() {
                let file_name = entry.file_name();
                let name = file_name.to_str();
                if let Some(name) = name {
                    let counter: Counter = name.parse()?;
                    next = next.max(counter + 1);
                }
            }
        }

        Ok(Box::new(Self { path, next }))
    }
    /// Creates a persister of updates, from the given path create a network subdirectory with
    /// another subdirectory which name is one-way derived from the descriptor
    pub fn new_with_desc<P: AsRef<Path>>(
        path: P,
        network: ElementsNetwork,
        desc: &WolletDescriptor,
    ) -> Result<Box<Self>, Error> {
        let mut persister_path = path.as_ref().to_path_buf();
        persister_path.push(network.as_str());
        persister_path.push(sha256::Hash::hash(desc.to_string().as_bytes()).to_string());
        Self::new(persister_path)
    }

    fn path(&self, counter: &Counter) -> PathBuf {
        let mut path = self.path.clone();
        path.push(counter.to_string());
        path
    }

    fn read(&self, current: usize) -> Result<Update, Error> {
        let path = self.path(&Counter::from(current));
        let bytes = fs::read(path)?;
        Ok(Update::deserialize(&bytes)?)
    }

    /// Write at next position without incrementing the next counter
    /// returns the number of bytes written
    fn write(&mut self, update: Update) -> Result<usize, Error> {
        let path = self.path(&self.next);
        let bytes = update.serialize()?;
        fs::write(path, &bytes)?;
        Ok(bytes.len())
    }
}

struct FsPersisterIter<'a> {
    current: usize,
    persister: &'a FsPersister,
}
impl<'a> Iterator for FsPersisterIter<'a> {
    type Item = Result<Update, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = usize::from(&self.persister.next);
        if self.current < next {
            let update = self.persister.read(self.current);
            match update {
                Ok(update) => {
                    self.current += 1;
                    Some(Ok(update))
                }
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = usize::from(&self.persister.next);
        (l, Some(l))
    }
}
impl<'a> ExactSizeIterator for FsPersisterIter<'a> {}

impl Persister for FsPersister {
    fn iter(&self) -> Box<dyn ExactSizeIterator<Item = Result<Update, Error>> + '_> {
        Box::new(FsPersisterIter {
            current: 0,
            persister: self,
        })
    }

    fn push(&mut self, update: Update) -> Result<usize, Error> {
        let _ = self.write(update)?;
        self.next = self.next.clone() + 1;
        Ok((&self.next).into())
    }
}

const PERSISTED_FILE_NAME_LENGTH: usize = 12;

/// Encapsulate an usize so that its to/from string representation are coherent
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
struct Counter(usize);

impl Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>width$}", self.0, width = PERSISTED_FILE_NAME_LENGTH)
    }
}
impl FromStr for Counter {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != PERSISTED_FILE_NAME_LENGTH {
            return Err(Error::Generic("Not 12 chars".to_string()));
        }
        let c: usize = s.parse()?;
        Ok(Self(c))
    }
}
impl From<usize> for Counter {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<&Counter> for usize {
    fn from(value: &Counter) -> Self {
        value.0
    }
}
impl From<Counter> for usize {
    fn from(value: Counter) -> Self {
        value.0
    }
}
impl Add<usize> for Counter {
    type Output = Counter;

    fn add(self, rhs: usize) -> Self::Output {
        Counter(rhs + self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::{Error, Update};

    use super::{Counter, FsPersister, NoPersist, Persister};

    struct MemoryPersister(Vec<Update>);
    impl MemoryPersister {
        pub fn new() -> Box<Self> {
            Box::new(Self(vec![]))
        }
    }
    impl Persister for MemoryPersister {
        fn iter(&self) -> Box<dyn ExactSizeIterator<Item = Result<Update, Error>> + '_> {
            Box::new(self.0.iter().map(|e| Ok(e.clone())))
        }

        fn push(&mut self, update: crate::Update) -> Result<usize, Error> {
            self.0.push(update);
            Ok(self.0.len())
        }
    }

    fn inner_test_persister(mut persister: Box<dyn Persister>, first_time: bool) {
        if first_time {
            assert_eq!(persister.iter().len(), 0);
        }

        let update1 = Update::deserialize(&lwk_test_util::update_test_vector_bytes()).unwrap();
        let update2 = {
            let mut update2 = update1.clone();
            update2.timestamps.push((22, 55));
            update2
        };
        assert_ne!(&update1, &update2);

        if first_time {
            persister.push(update1.clone()).unwrap();
            let mut iter = persister.iter();
            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next().unwrap().unwrap(), update1.clone());
            assert!(iter.next().is_none());
            drop(iter);

            persister.push(update2.clone()).unwrap();
        }
        let mut iter = persister.iter();
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next().unwrap().unwrap(), update1);
        assert_eq!(iter.next().unwrap().unwrap(), update2);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_memory_persister() {
        let persister = MemoryPersister::new();
        inner_test_persister(persister, true);
    }

    #[test]
    fn test_no_persist() {
        let mut persister = NoPersist {};
        assert_eq!(persister.iter().len(), 0);
        let update = Update::deserialize(&lwk_test_util::update_test_vector_bytes()).unwrap();
        persister.push(update).unwrap();
        assert_eq!(persister.iter().len(), 0);
    }

    #[test]
    fn test_fs_persister() {
        let tempdir = tempfile::tempdir().unwrap();
        let persister = FsPersister::new(&tempdir).unwrap();
        inner_test_persister(persister, true);
        let persister = FsPersister::new(&tempdir).unwrap();
        inner_test_persister(persister, false);
    }

    #[test]
    fn test_counter() {
        let c = Counter::default();
        assert_eq!(c.to_string(), "000000000000");
        assert_eq!(usize::from(c), 0);
        assert_eq!(Counter::from(100).to_string(), "000000000100");
    }
}