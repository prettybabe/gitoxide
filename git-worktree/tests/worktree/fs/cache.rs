mod create_directory {

    use std::path::Path;

    use git_index::entry::Mode;
    use git_worktree::fs;
    use tempfile::{tempdir, TempDir};

    #[test]
    fn root_is_assumed_to_exist_and_files_in_root_do_not_create_directory() {
        let dir = tempdir().unwrap();
        let mut cache = fs::Cache::new(
            dir.path().join("non-existing-root"),
            fs::cache::Options {
                create_directories: true,
                ..Default::default()
            },
        );
        assert_eq!(cache.test_mkdir_calls, 0);

        let path = cache
            .append_relative_path_assure_leading_dir("hello", Mode::FILE)
            .unwrap();
        assert!(!path.parent().unwrap().exists(), "prefix itself is never created");
        assert_eq!(cache.test_mkdir_calls, 0);
    }

    #[test]
    fn directory_paths_are_created_in_full() {
        let (mut cache, _tmp) = new_cache();

        for (name, mode) in &[
            ("dir", Mode::DIR),
            ("submodule", Mode::COMMIT),
            ("file", Mode::FILE),
            ("exe", Mode::FILE_EXECUTABLE),
            ("link", Mode::SYMLINK),
        ] {
            let path = cache
                .append_relative_path_assure_leading_dir(Path::new("dir").join(name), *mode)
                .unwrap();
            assert!(path.parent().unwrap().is_dir(), "dir exists");
        }

        assert_eq!(cache.test_mkdir_calls, 3);
    }

    #[test]
    fn existing_directories_are_fine() {
        let (mut cache, tmp) = new_cache();
        std::fs::create_dir(tmp.path().join("dir")).unwrap();

        let path = cache
            .append_relative_path_assure_leading_dir("dir/file", Mode::FILE)
            .unwrap();
        assert!(path.parent().unwrap().is_dir(), "directory is still present");
        assert!(!path.exists(), "it won't create the file");
        assert_eq!(cache.test_mkdir_calls, 1);
    }

    #[test]
    fn symlinks_or_files_in_path_are_forbidden_or_unlinked_when_forced() {
        let (mut cache, tmp) = new_cache();
        let forbidden = tmp.path().join("forbidden");
        std::fs::create_dir(&forbidden).unwrap();
        symlink::symlink_dir(&forbidden, tmp.path().join("link-to-dir")).unwrap();
        std::fs::write(tmp.path().join("file-in-dir"), &[]).unwrap();

        for dirname in &["file-in-dir", "link-to-dir"] {
            cache.unlink_on_collision = false;
            let relative_path = format!("{}/file", dirname);
            assert_eq!(
                cache
                    .append_relative_path_assure_leading_dir(&relative_path, Mode::FILE)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::AlreadyExists
            );
        }
        assert_eq!(
            cache.test_mkdir_calls, 2,
            "it tries to create each directory once, but it's a file"
        );
        cache.test_mkdir_calls = 0;
        for dirname in &["link-to-dir", "file-in-dir"] {
            cache.unlink_on_collision = true;
            let relative_path = format!("{}/file", dirname);
            let path = cache
                .append_relative_path_assure_leading_dir(&relative_path, Mode::FILE)
                .unwrap();
            assert!(path.parent().unwrap().is_dir(), "directory was forcefully created");
            assert!(!path.exists());
        }
        assert_eq!(
            cache.test_mkdir_calls, 4,
            "like before, but it unlinks what's there and tries again"
        );
    }

    fn new_cache() -> (fs::Cache, TempDir) {
        let dir = tempdir().unwrap();
        let cache = fs::Cache::new(
            dir.path(),
            fs::cache::Options {
                create_directories: true,
                ..Default::default()
            },
        );
        (cache, dir)
    }
}
