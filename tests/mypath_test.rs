extern crate wormhole;
use crate::wormhole::providers::mypath::{pathType, MyPath};

#[test]
fn test_mypath_new() {
    let path = MyPath::new("./StarWars/A_New_Hope");
    assert_eq!(
        path,
        MyPath {
            inner: String::from("./StarWars/A_New_Hope"),
            kind: pathType::Relative
        }
    );

    let path = MyPath::new("/HarryPotter/The_Goblet_of_Fire");
    assert_eq!(
        path,
        MyPath {
            inner: String::from("/HarryPotter/The_Goblet_of_Fire"),
            kind: pathType::Absolute
        }
    );

    let path = MyPath::new("TheLordofTheRings/The_Two_Towers/");
    assert_eq!(
        path,
        MyPath {
            inner: String::from("TheLordofTheRings/The_Two_Towers/"),
            kind: pathType::NoPrefix
        }
    );

    let path = MyPath::new("");
    assert_eq!(
        path,
        MyPath {
            inner: String::from(""),
            kind: pathType::Empty
        }
    );
}

fn mypath_join(mut path: MyPath, join: &str, result: &MyPath) {
    let new_path = path.join(join);
    assert_eq!(new_path, result);
}

#[test]
fn test_mypath_join_with_str() {
    let path1 = MyPath::new("Kaamelott");
    let path2 = MyPath::new("./Kaamelott/");

    mypath_join(
        path1.clone(),
        "./Season 1/",
        &MyPath::new("Kaamelott/Season 1/"),
    );
    mypath_join(
        path1.clone(),
        "/Season 2/",
        &MyPath::new("Kaamelott/Season 2/"),
    );
    mypath_join(
        path1.clone(),
        "Season 3/",
        &MyPath::new("Kaamelott/Season 3/"),
    );
    mypath_join(
        path1.clone(),
        "Season 4",
        &MyPath::new("Kaamelott/Season 4"),
    );

    mypath_join(
        path2.clone(),
        "./Season 1/",
        &MyPath::new("./Kaamelott/Season 1/"),
    );
    mypath_join(
        path2.clone(),
        "/Season 2/",
        &MyPath::new("./Kaamelott/Season 2/"),
    );
    mypath_join(
        path2.clone(),
        "Season 3/",
        &MyPath::new("./Kaamelott/Season 3/"),
    );
    mypath_join(
        path2.clone(),
        "Season 4",
        &MyPath::new("./Kaamelott/Season 4"),
    );
}
