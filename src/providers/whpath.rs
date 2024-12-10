#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    Absolute,
    Relative,
    NoPrefix,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhPath {
    pub inner: String,
    pub kind: PathType,
}

impl WhPath {
    pub fn new<S: AsRef<str>>(path: S) -> Self {
        let p = String::from(path.as_ref());
        let mut pth = WhPath {
            inner: p.clone(),
            kind: PathType::Empty,
        };
        pth.update_kind();
        pth
    }

    //TODO - Faire un join pour de WhPath
    //NOTE - join deux paths dans l'ordre indiqué, résoud le conflit si le second commence avec ./ ou / ou rien
    pub fn join<S: AsRef<str>>(&mut self, segment: S) -> &Self {
        self.add_last_slash();
        let seg = Self::remove_leading_slash(segment.as_ref());
        self.inner = format!("{}{}", self.inner, seg);
        self
    }

    //NOTE - retire la partie demandée "/my/file/path/".remove("file/path") = "/my/"
    pub fn remove<S: AsRef<str>>(&mut self, delete_this_part: S) -> &Self {
        self.inner = self.inner.replace(delete_this_part.as_ref(), "");
        self.delete_double_slash();
        self.convert_path(self.kind.clone());
        self
    }

    //NOTE - Modifier le path pour que celui corresponde au nouveau nom demandé
    // Ne peut modifier que le dernier élément du path
    pub fn rename<S: AsRef<str>>(&mut self, file_name: S) -> &Self {
        if let Some(pos) = self.inner.rfind('/') {
            if pos == self.inner.len() - 1 {
                // self.inner = Self::remove_last_slash(self.inner.clone());
                self.inner.pop();
                self.rename(file_name);
                self.inner.push('/');
                return self;
            }
            self.inner = format!("{}/{}", &self.inner[..pos], file_name.as_ref());
        } else {
            self.inner = file_name.as_ref().to_string();
        }
        self.update_kind();
        self
    }

    pub fn kind(&self) -> PathType {
        if self.is_empty() {
            return PathType::Empty;
        }
        if self.inner.chars().next() == Some('.') {
            return PathType::Relative;
        } else if self.inner.chars().next() == Some('/') {
            return PathType::Absolute;
        } else {
            return PathType::NoPrefix;
        }
    }

    pub fn update_kind (&mut self) {
        self.kind = self.kind();
    }

    //NOTE - changer le path pour "./path"
    pub fn set_relative(&mut self) -> &Self {
        if !self.is_empty() && !Self::is_relative(&self) {
            self.convert_path(PathType::Relative);
        }
        self
    }

    //NOTE - changer le path pour "/path"
    pub fn set_absolute(&mut self) -> &Self {
        if !self.is_empty() && !Self::is_absolute(&self) {
            self.convert_path(PathType::Absolute);
        }
        self
    }

    //NOTE - changer le path pour "path"
    pub fn remove_prefix(&mut self) -> &Self {
        if !self.is_empty() && !Self::has_no_prefix(&self) {
            self.convert_path(PathType::NoPrefix);
        }
        self
    }

    pub fn is_relative(&self) -> bool {
        self.kind == PathType::Relative
    }

    pub fn is_absolute(&self) -> bool {
        self.kind == PathType::Absolute
    }

    pub fn has_no_prefix(&self) -> bool {
        self.kind == PathType::NoPrefix
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    //NOTE - fonctions pour mettre ou non un / à la fin
    pub fn set_end(&mut self, end: bool) -> &Self {
        if end {
            self.add_last_slash();
        } else {
            self.remove_last_slash();
        };
        self
    }

    //NOTE - true si le path demandé est dans le path original (comme tu gères des string c'est un startwith, en gros)
    pub fn is_in<S: AsRef<str>>(&self, segment: S) -> bool {
        self.inner.starts_with(segment.as_ref())
    }

    //NOTE - donne le dernier élément du path
    pub fn get_end(&self) -> String {
        let mut path = self.clone();
        path.remove_last_slash();
        match path.inner.rsplit('/').next() {
            Some(last) => last.to_string(),
            _none => String::from(""),
        }
    }

    pub fn remove_end(&mut self) -> &Self {
        self.remove_last_slash();
        if let Some(pos) = self.inner.rfind('/') {
            self.inner = self.inner[..(pos + 1)].to_string();
        } else {
            self.convert_path(PathType::Empty);
        }
        return self;
    }

    pub fn to_vector(&mut self) -> Vec<String> {
        let mut elements: Vec<String> = vec![];

        while !self.is_empty() {
            elements.push(self.get_end());
            self.remove_end();
        }
        let elements = elements.into_iter().rev().collect();
        elements
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    ///!SECTION- A modifier pour prendre en compte les fichiers cachés ?
    fn remove_leading_slash(segment: &str) -> &str {
        let mut i = 0;
        for c in segment.chars() {
            if c == '.' || c == '/' {
                i += 1;
            } else {
                break;
            }
        }
        return &segment[i..];
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    fn add_last_slash(&mut self) -> &Self {
        if self.kind != PathType::Empty && self.inner.chars().last() != Some('/') {
            self.inner = format!("{}/", self.inner);
        }
        return self;
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    fn remove_last_slash(&mut self) -> &Self {
        if let Some(pos) = self.inner.rfind('/') {
            if pos == self.inner.len() - 1 {
                self.inner.pop();
            }
        }
        return self;
    }

    fn delete_double_slash(&mut self) -> &Self {
        let mut i = 0;
        let mut index = 0;
        while index < self.inner.len() {
            i = if self.inner.as_bytes()[index] == b'/' {
                i + 1
            } else {
                0
            };
            if i == 2 {
                self.inner.remove(index);
                i -= 1;
                continue;
            }

            index += 1;
        }
        return self;
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    fn convert_path(&mut self, pathtype: PathType) -> &Self {
        if pathtype == PathType::Empty || self.inner == String::from("") {
            self.inner = String::from("");
            self.kind = PathType::Empty;
            return self;
        }
        self.inner = Self::remove_leading_slash(&self.inner.clone()).to_string();
        if pathtype == PathType::Absolute {
            self.inner = format!("/{}", self.inner);
        }
        if pathtype == PathType::Relative {
            self.inner = format!("./{}", self.inner);
        } else {
        }
        self.update_kind();
        return self;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_whpath_remove_leading_slash() {
        assert_eq!(WhPath::remove_leading_slash("bar"), "bar");
        assert_eq!(WhPath::remove_leading_slash("./bar"), "bar");
        assert_eq!(WhPath::remove_leading_slash("/bar"), "bar");
        assert_eq!(WhPath::remove_leading_slash(""), "");
        assert_eq!(WhPath::remove_leading_slash(".bar"), "bar");
    }

    #[test]
    fn test_whpath_add_last_slash() {
        let mut empty = WhPath::new("");
        let mut basic_folder = WhPath::new("foo/");
        let mut no_slash = WhPath::new("baz");

        assert_eq!(empty.add_last_slash(), &WhPath::new(""));
        assert_eq!(basic_folder.add_last_slash(), &WhPath::new("foo/"));
        assert_eq!(no_slash.add_last_slash(), &WhPath::new("baz/"));
    }

    #[test]
    fn test_whpath_remove_last_slash() {
        let mut empty = WhPath::new("");
        let mut basic_folder = WhPath::new("foo/");
        let mut no_slash = WhPath::new("baz/bar");

        assert_eq!(empty.remove_last_slash(), &WhPath::new(""));
        assert_eq!(basic_folder.remove_last_slash(), &WhPath::new("foo"));
        assert_eq!(no_slash.remove_last_slash(), &WhPath::new("baz/bar"));
    }

    #[test]
    fn test_whpath_delete_double_slash() {
        let mut empty = WhPath::new("");
        let mut basic_folder = WhPath::new("foo/");
        let mut mid_double_slash = WhPath::new("baz//bar");
        let mut end_double_slash = WhPath::new("baz/bar//");

        assert_eq!(empty.delete_double_slash(), &WhPath::new(""));
        assert_eq!(basic_folder.delete_double_slash(), &WhPath::new("foo/"));
        assert_eq!(
            mid_double_slash.delete_double_slash(),
            &WhPath::new("baz/bar")
        );
        assert_eq!(
            end_double_slash.delete_double_slash(),
            &WhPath::new("baz/bar/")
        );
    }

    #[test]
    fn test_whpath_convert_path() {
        let mut path = WhPath::new("foo");
        let mut relative = WhPath::new("./foo");
        let mut no_prefix = WhPath::new("foo");
        let mut absolute = WhPath::new("/foo");
        let mut empty = WhPath::new("");

        assert_eq!(relative.convert_path(PathType::NoPrefix), &no_prefix);
        assert_eq!(no_prefix.convert_path(PathType::Absolute), &absolute);
        assert_eq!(absolute.convert_path(PathType::Empty), &empty);
        assert_eq!(empty.convert_path(PathType::Absolute), &WhPath::new(""));
        assert_eq!(path.convert_path(PathType::Relative), &WhPath::new("./foo"));
    }

    #[test]
    fn test_whpath_to_vector() {
        let mut path = WhPath::new("foo/pouet/lol");

        assert_eq!(path.to_vector(), vec!["foo", "pouet", "lol"]);
    }
}