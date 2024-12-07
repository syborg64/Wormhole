#[derive(Debug, Clone, PartialEq)]
pub enum pathType {
    Absolute,
    Relative,
    NoPrefix,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhPath {
    pub inner: String,
    pub kind: pathType,
}

impl WhPath {
    pub fn new<S: AsRef<str>>(path: S) -> Self {
        let p = String::from(path.as_ref());
        let kind = WhPath {
            inner: p.clone(),
            kind: pathType::Empty,
        }
        .kind();
        WhPath {
            inner: p,
            kind: kind,
        }
    }

    //TODO - Faire un join pour de WhPath
    //NOTE - join deux paths dans l'ordre indiqué, résoud le conflit si le second commence avec ./ ou / ou rien
    pub fn join<S: AsRef<str>>(&mut self, segment: S) -> &Self {
        self.inner =
            Self::add_last_slash(self.inner.clone()) + Self::remove_leading_slash(segment.as_ref());
        return self;
    }

    //NOTE - retire la partie demandée "/my/file/path/".remove("file/path") = "/my/"
    pub fn remove<S: AsRef<str>>(&mut self, delete_this_part: S) -> &Self {
        self.inner = self.inner.replace(delete_this_part.as_ref(), "");
        self.delete_double_slash();
        if self.is_empty() {
            self.kind = pathType::Empty;
        }
        self.inner = Self::convert_path(&self.inner.clone(), self.kind.clone());
        return self;
    }

    //NOTE - Modifier le path pour que celui corresponde au nouveau nom demandé
    pub fn rename<S: AsRef<str>>(&mut self, file_name: S) -> &Self {
        self.inner = self.remove_end() + file_name.as_ref();
        return self;
    }

    pub fn kind(self) -> pathType {
        if self.is_empty() {
            return pathType::Empty;
        }
        if self.inner.chars().next() == Some('.') {
            return pathType::Relative;
        } else if self.inner.chars().next() == Some('/') {
            return pathType::Absolute;
        } else {
            return pathType::NoPrefix;
        }
    }

    //NOTE - changer le path pour "./path"
    pub fn setRelative(&mut self) -> &Self {
        if !self.is_empty() && !Self::is_relative(&self) {
            self.inner = Self::convert_path(&self.inner, pathType::Relative);
            self.kind = pathType::Relative;
        }
        return self;
    }

    //NOTE - changer le path pour "/path"
    pub fn set_absolute(&mut self) -> &Self {
        if !self.is_empty() && !Self::is_absolute(&self) {
            self.inner = Self::convert_path(&self.inner, pathType::Absolute);
            self.kind = pathType::Absolute;
        }
        return self;
    }

    //NOTE - changer le path pour "path"
    pub fn remove_prefix(&mut self) -> &Self {
        if !self.is_empty() && !Self::has_no_prefix(&self) {
            self.inner = Self::convert_path(&self.inner, pathType::NoPrefix);
            self.kind = pathType::NoPrefix;
        }
        return self;
    }

    pub fn is_relative(&self) -> bool {
        return self.kind == pathType::Relative;
    }

    pub fn is_absolute(&self) -> bool {
        return self.kind == pathType::Absolute;
    }

    pub fn has_no_prefix(&self) -> bool {
        return self.kind == pathType::NoPrefix;
    }

    pub fn is_empty(&self) -> bool {
        return self.inner.is_empty();
    }

    //NOTE - fonctions pour mettre ou non un / à la fin
    pub fn set_end(&mut self, end: bool) -> &Self {
        self.inner = if end {
            Self::add_last_slash(self.inner.clone())
        } else {
            Self::remove_last_slash(self.inner.clone())
        };
        return self;
    }

    //NOTE - true si le path demandé est dans le path original (comme tu gères des string c'est un startwith, en gros)
    pub fn isln<S: AsRef<str>>(&self, segment: S) -> bool {
        return self.inner.starts_with(segment.as_ref());
    }

    //NOTE - donne le dernier élément du path
    pub fn get_end(&self) -> String {
        let str = Self::remove_last_slash(self.inner.clone());
        match str.rsplit('/').next() {
            Some(last) => last.to_string(),
            _none => String::from(""),
        }
    }

    pub fn remove_end(&self) -> String {
        let str = Self::remove_last_slash(self.inner.clone());
        match str.rfind('/') {
            Some(pos) => self.inner[..pos].to_string(),
            _none => String::from(""),
        }
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
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
    fn add_last_slash(segment: String) -> String {
        if segment.chars().last() != Some('/') {
            return segment + "/";
        }
        return segment;
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    fn remove_last_slash(segment: String) -> String {
        if segment.chars().last() == Some('/') {
            return segment.trim_end().to_string();
        }
        return segment;
    }

    fn delete_double_slash(&mut self) {
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
    }

    ///!SECTION - Est-ce qu'il faudra modifier pour Windows en rajoutant le '\' ??
    fn convert_path(segment: &str, pathtype: pathType) -> String {
        if pathtype == pathType::Empty {
            return String::from("");
        }
        let seg = Self::remove_leading_slash(segment);
        if pathtype == pathType::Absolute {
            return "/".to_string() + seg;
        } else if pathtype == pathType::Relative {
            return "./".to_string() + seg;
        } else {
            return seg.to_string();
        }
    }
}
