extern crate wormhole;
use crate::wormhole::providers::whpath::{pathType, WhPath};

#[test]
fn test_whpath_new() {
    let path = WhPath::new("./StarWars/A_New_Hope");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("./StarWars/A_New_Hope"),
            kind: pathType::Relative
        }
    );

    let path = WhPath::new("/HarryPotter/The_Goblet_of_Fire");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("/HarryPotter/The_Goblet_of_Fire"),
            kind: pathType::Absolute
        }
    );

    let path = WhPath::new("TheLordofTheRings/The_Two_Towers/");
    assert_eq!(
        path,
        WhPath {
            inner: String::from("TheLordofTheRings/The_Two_Towers/"),
            kind: pathType::NoPrefix
        }
    );

    let path = WhPath::new("");
    assert_eq!(
        path,
        WhPath {
            inner: String::from(""),
            kind: pathType::Empty
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
