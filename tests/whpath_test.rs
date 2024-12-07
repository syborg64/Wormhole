extern crate wormhole;
use crate::wormhole::providers::whpath::{PathType, WhPath};

#[test]
fn test_whpath_kind() {
    let empty = WhPath {
        inner: String::from(""),
        kind: PathType::Empty,
    };
    let absolute = WhPath {
        inner: String::from("/foo"),
        kind: PathType::Empty,
    };
    let relative = WhPath {
        inner: String::from("./foo"),
        kind: PathType::Empty,
    };
    let noprefix = WhPath {
        inner: String::from("foo"),
        kind: PathType::Empty,
    };

    assert_eq!(empty.kind(), PathType::Empty);
    assert_eq!(absolute.kind(), PathType::Absolute);
    assert_eq!(relative.kind(), PathType::Relative);
    assert_eq!(noprefix.kind(), PathType::NoPrefix);
}

#[test]
fn test_whpath_new() {
    let path = WhPath::new("./StarWars/A_New_Hope");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("./StarWars/A_New_Hope"),
            kind: PathType::Relative
        }
    );

    let path = WhPath::new("/HarryPotter/The_Goblet_of_Fire");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("/HarryPotter/The_Goblet_of_Fire"),
            kind: PathType::Absolute
        }
    );

    let path = WhPath::new("TheLordofTheRings/The_Two_Towers/");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("TheLordofTheRings/The_Two_Towers/"),
            kind: PathType::NoPrefix
        }
    );

    let path = WhPath::new("");
    assert_eq!(
        path,
        WhPath {
            inner: String::from(""),
            kind: PathType::Empty
        }
    );
}

fn whpath_join(mut path: WhPath, join: &str, result: &WhPath) {
    let new_path = path.join(join);
    assert_eq!(new_path, result);
}

#[test]
fn test_whpath_join_with_str() {
    let path1 = WhPath::new("Kaamelott");
    let path2 = WhPath::new("./Kaamelott/");

    whpath_join(
        path1.clone(),
        "./Season 1/",
        &WhPath::new("Kaamelott/Season 1/"),
    );
    whpath_join(
        path1.clone(),
        "/Season 2/",
        &WhPath::new("Kaamelott/Season 2/"),
    );
    whpath_join(
        path1.clone(),
        "Season 3/",
        &WhPath::new("Kaamelott/Season 3/"),
    );
    whpath_join(
        path1.clone(),
        "Season 4",
        &WhPath::new("Kaamelott/Season 4"),
    );

    whpath_join(path1.clone(), "", &WhPath::new("Kaamelott/"));

    whpath_join(
        path2.clone(),
        "./Season 1/",
        &WhPath::new("./Kaamelott/Season 1/"),
    );
    whpath_join(
        path2.clone(),
        "/Season 2/",
        &WhPath::new("./Kaamelott/Season 2/"),
    );
    whpath_join(
        path2.clone(),
        "Season 3/",
        &WhPath::new("./Kaamelott/Season 3/"),
    );
    whpath_join(
        path2.clone(),
        "Season 4",
        &WhPath::new("./Kaamelott/Season 4"),
    );

    whpath_join(path2.clone(), "", &WhPath::new("./Kaamelott/"));
}

#[test]
fn test_whpath_remove() {
    let mut delete_all = WhPath::new("Film/Orson Welles/Citizen Kane");
    let mut delete_end = WhPath::new("Film/Orson Welles/The Magnificent Ambersons");
    let mut delete_middle = WhPath::new("Film/Orson Welles/The Lady from Shanghai");
    let mut delete_start = WhPath::new("Film/Orson Welles/Touch of Evil");
    let mut error = WhPath::new("Film/Orson Welles/Chimes at Midnight");

    assert_eq!(
        delete_all.remove("Film/Orson Welles/Citizen Kane"),
        &WhPath::new("")
    );
    assert_eq!(
        delete_end.remove("The Magnificent Ambersons"),
        &WhPath::new("Film/Orson Welles/")
    );

    assert_eq!(
        delete_middle.remove("Orson Welles"),
        &WhPath::new("Film/The Lady from Shanghai")
    );
    assert_eq!(
        delete_start.remove("Film"),
        &WhPath::new("Orson Welles/Touch of Evil")
    );

    assert_eq!(
        error.remove("Series"),
        &WhPath::new("Film/Orson Welles/Chimes at Midnight")
    );
}

#[test]
fn test_whpath_rename() {
    let mut empty = WhPath::new("");
    let mut basic_folder = WhPath::new("./foo/");
    let mut basic_file = WhPath::new("foo/file.txt");
    let mut basic_subfolder = WhPath::new("foo/bar/");

    assert_eq!(empty.rename("baz"), &WhPath::new("baz"));
    assert_eq!(basic_folder.rename("bar"), &WhPath::new("./bar/"));
    assert_eq!(basic_file.rename("foo.rs"), &WhPath::new("foo/foo.rs"));
    assert_eq!(basic_subfolder.rename("sub"), &WhPath::new("foo/sub/"));
}

#[test]
fn test_whpath_set_relative() {
    let mut no_prefix = WhPath::new("foo");
    let mut absolute = WhPath::new("/foo/");
    let mut empty = WhPath::new("");

    assert_eq!(no_prefix.set_relative(), &WhPath::new("./foo"));
    assert_eq!(absolute.set_relative(), &WhPath::new("./foo/"));
    assert_eq!(empty.set_relative(), &WhPath::new(""));
}

#[test]
fn test_whpath_set_absolute() {
    let mut no_prefix = WhPath::new("foo");
    let mut relative = WhPath::new("./foo/");
    let mut empty = WhPath::new("");

    assert_eq!(no_prefix.set_absolute(), &WhPath::new("/foo"));
    assert_eq!(relative.set_absolute(), &WhPath::new("/foo/"));
    assert_eq!(empty.set_absolute(), &WhPath::new(""));
}

#[test]
fn test_whpath_remove_prefix() {
    let mut absolute = WhPath::new("/foo");
    let mut relative = WhPath::new("./foo/");
    let mut empty = WhPath::new("");

    assert_eq!(absolute.remove_prefix(), &WhPath::new("foo"));
    assert_eq!(relative.remove_prefix(), &WhPath::new("foo/"));
    assert_eq!(empty.remove_prefix(), &WhPath::new(""));
}

#[test]
fn test_whpath_is_relative() {
    let relative = WhPath::new("./foo");
    let not_relative = WhPath::new("foo");

    assert_eq!(relative.is_relative(), true);
    assert_eq!(not_relative.is_relative(), false);
}

#[test]
fn test_whpath_is_absolute() {
    let absolute = WhPath::new("/foo");
    let not_absolute = WhPath::new("foo");

    assert_eq!(absolute.is_absolute(), true);
    assert_eq!(not_absolute.is_absolute(), false);
}
#[test]
fn test_whpath_has_no_prefix() {
    let no_prefix = WhPath::new("foo");
    let not_no_prefix = WhPath::new("/foo");

    assert_eq!(no_prefix.has_no_prefix(), true);
    assert_eq!(not_no_prefix.has_no_prefix(), false);
}
#[test]
fn test_whpath_is_empty() {
    let empty = WhPath::new("");
    let not_empty = WhPath::new("foo");

    assert_eq!(empty.is_empty(), true);
    assert_eq!(not_empty.is_empty(), false);
}

#[test]
fn test_whpath_set_end() {
    let mut set_end_true = WhPath::new("/foo");
    let mut set_end_false = WhPath::new("/foo/");
    assert_eq!(set_end_true.set_end(true), &WhPath::new("/foo/"));
    assert_eq!(set_end_false.set_end(false), &WhPath::new("/foo"));
}

#[test]
fn test_whpath_isln() {
    let path = WhPath::new("/foo/bar/baz.txt");
    let empty = WhPath::new("");
    assert_eq!(path.isln("/foo/bar"), true);
    assert_eq!(path.isln("bar"), false);
    assert_eq!(path.isln("peanuts"), false);
    assert_eq!(empty.isln("peanuts"), false);
    assert_eq!(empty.isln(""), true);
}
