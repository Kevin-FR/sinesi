use sns_vfs::*;
#[test]
fn r_files_isolation_copy_move_remove_and_binary_round_trip() {
    let mut a = Vfs::default();
    let b = Vfs::default();
    a.mkdir("/exercice").unwrap();
    a.write(
        "/exercice/index.html",
        "Bonjour · ¡Hola! · Olá".as_bytes(),
        "text/html",
        false,
    )
    .unwrap();
    let image = include_bytes!("../../sns-ui-egui/tests/snapshots/j1-editor.png");
    a.write("/exercice/image.png", image, "image/png", false)
        .unwrap();
    assert!(!b.exists("/exercice"));
    a.mkdir("/copie").unwrap();
    a.copy("/exercice", "/copie").unwrap();
    a.move_entry("/copie/exercice/index.html", "/copie/exercice/renomme.html")
        .unwrap();
    assert!(!a.exists("/copie/exercice/index.html"));
    assert!(
        a.read_text("/copie/exercice/renomme.html")
            .unwrap()
            .contains("¡Hola!")
    );
    let restored = Vfs::from_files(&a.to_files()).unwrap();
    assert_eq!(restored, a);
    assert_eq!(restored.read("/exercice/image.png").unwrap(), image);
    assert_eq!(
        a.remove("/copie", false).unwrap_err().code,
        "fs.directory_not_empty"
    );
    a.remove("/copie", true).unwrap();
    assert!(a.exists("/exercice/image.png"));
    assert!(!a.exists("/copie"));
}
#[test]
fn normalization_never_reaches_host_and_failures_preserve_tree() {
    assert_eq!(resolve("/a/b", "../c/./note.txt").unwrap(), "/a/c/note.txt");
    assert_eq!(resolve("/", "../../..").unwrap(), "/");
    for bad in [
        "file:///etc/passwd",
        "https://host/a",
        "C:\\file",
        "//host/share",
        "/%2e%2e/secrets",
    ] {
        assert!(resolve("/", bad).is_err());
    }
    let mut v = Vfs::default();
    v.mkdir("/a").unwrap();
    v.write("/a/f", b"text", "text/plain", false).unwrap();
    let original = v.clone();
    assert!(v.copy("/a", "/a/child").is_err());
    assert_eq!(v, original);
    assert!(v.move_entry("/a", "/a/child").is_err());
    assert_eq!(v, original);
    assert!(v.write("/a/f", b"replace", "text/plain", false).is_err());
    assert_eq!(v, original);
    assert!(v.remove("/", true).is_err());
    assert_eq!(v, original);
}
#[test]
fn legacy_implicit_directories_and_limits_are_explicit() {
    let files = [sns_model::VirtualFile {
        path: "/images/logo.png".into(),
        media_type: "image/png".into(),
        data_base64: String::new(),
    }];
    let mut v = Vfs::from_files(&files).unwrap();
    assert!(v.is_directory("/images"));
    let original = v.clone();
    assert!(
        v.write("/large", &vec![0; MAX_HOST_BYTES + 1], "image/png", false)
            .is_err()
    );
    assert_eq!(v, original);
    v.write("/large", &vec![0; MAX_HOST_BYTES], "image/png", false)
        .unwrap();
    assert!(v.write("/extra", b"x", "text/plain", false).is_err());
    assert!(!v.exists("/extra"));
}
