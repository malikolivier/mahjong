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
