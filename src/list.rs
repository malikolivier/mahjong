use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrderedList<T> {
    container: Vec<T>,
}

impl<T> Default for OrderedList<T> {
    fn default() -> Self {
        OrderedList {
            container: Vec::new(),
        }
    }
}

impl<T: PartialOrd> OrderedList<T> {
    pub fn insert(&mut self, element: T) {
        let mut index = 0;
        for item in &self.container {
            if item <= &element {
                index += 1;
                continue;
            } else {
                break;
            }
        }
        self.container.insert(index, element)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.container.iter()
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.container.remove(index)
    }

    pub fn contains(&self, element: &T) -> bool {
        self.container.contains(element)
    }

    pub fn index(&self, element: &T) -> Option<usize> {
        self.container.iter().position(|x| x == element)
    }
}

impl<'x, T: 'x + PartialOrd> OrderedList<T> {
    pub fn contains_all<I: IntoIterator<Item = &'x T>>(&self, ls: I) -> bool {
        for x in ls {
            if !self.contains(&x) {
                return false;
            }
        }
        true
    }
}

impl<T: Serialize> Serialize for OrderedList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.container.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for OrderedList<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let container: Vec<T> = Vec::deserialize(deserializer)?;
        Ok(Self { container })
    }
}
