use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum pathType {
    Absolute,
    Relative,
    NoPrefix,
}

#[derive(Debug, Clone)]
struct MyPath {
    inner: String,
    typ: pathType,
}

impl MyPath {
    pub fn new<S: AsRef<str>>(path: S, typ: pathType) -> Self {
        MyPath {
            inner: String::from(path.as_ref()),
            typ: typ,
        }
    }

    //TODO - join deux paths dans l'ordre indiqué, résoud le conflit si le second commence avec ./ ou / ou rien
    pub fn join<S: AsRef<str>>(&self, segment: S) -> Self {
        MyPath {
            inner: self.inner.clone() + segment.as_ref(),
            typ: self.typ.clone(),
        }
    }

    //TODO - retire la partie demandée "/my/file/path/".remove("file/path") = "/my/"
    pub fn remove(&self, delete_this_part: &str) -> Self {
        let _ = delete_this_part;
        MyPath {
            inner: self.inner.clone(),
            typ: self.typ.clone(),
        }
    }

    //TODO - Modifier le path pour que celui corresponde au nouveau nom demandé
    pub fn rename(&self, file_name: &str) -> Self {
        let _ = file_name;
        MyPath {
            inner: self.inner.clone(),
            typ: self.typ.clone(),
        }
    }

    //TODO - changer le path pour "./path"
    pub fn setRelative(&self) -> Self {
        MyPath {
            inner: self.inner.clone(),
            typ: self.typ.clone(),
        }
    }

    //TODO - changer le path pour "/path"
    //NOTE -  il ne faut pas just enlever le point mais mettre le path absolue de l'ordinateur
    pub fn setAbsolute(&self) -> Self {
        MyPath {
            inner: self.inner.clone(),
            typ: self.typ.clone(),
        }
    }

    //TODO - changer le path pour "path"
    pub fn removePrefix(&self) -> Self {
        MyPath {
            inner: self.inner.clone(),
            typ: self.typ.clone(),
        }
    }

    //TODO
    pub fn isRelative(&self) -> bool {
        return false;
    }

    //TODO
    pub fn isAbsolute(&self) -> bool {
        return false;
    }

    //TODO
    pub fn hasNoPrefix(&self) -> bool {
        return false;
    }

    //TODO - fonctions pour mettre ou non un / à la fin
    pub fn setEnd(&self, end: bool) -> Self {
        MyPath {
            inner: if end {self.inner.clone()} else {self.inner.clone()},
            typ: self.typ.clone(),
        }
    }

    //TODO - true si le path demandé est dans le path original (comme tu gères des string c'est un startwith, en gros)
    pub fn isln(&self) -> bool {
        return false;
    }

    //TODO - donne le dernier élément du path
    pub fn getEnd(&self) -> &str {
        return "test";
    }
}
