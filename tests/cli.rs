#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_basic_functionality() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        let file1_path = test_dir.join("file1.txt");
        let file2_path = test_dir.join("file2.txt");
        fs::File::create(&file1_path)
            .unwrap()
            .write_all(b"Contents of file1")
            .unwrap();
        fs::File::create(&file2_path)
            .unwrap()
            .write_all(b"Contents of file2")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test_dir/file1.txt"))
            .stdout(predicate::str::contains("Contents of file1"))
            .stdout(predicate::str::contains("test_dir/file2.txt"))
            .stdout(predicate::str::contains("Contents of file2"));
    }

    #[test]
    fn test_include_hidden() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        let hidden_file_path = test_dir.join(".hidden.txt");
        fs::File::create(&hidden_file_path)
            .unwrap()
            .write_all(b"Contents of hidden file")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(".hidden.txt").not());
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg("--include-hidden");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test_dir/.hidden.txt"))
            .stdout(predicate::str::contains("Contents of hidden file"));
    }


    #[test]
    fn test_multiple_paths() {
        let dir = tempdir().unwrap();
        let test_dir1 = dir.path().join("test_dir1");
        let test_dir2 = dir.path().join("test_dir2");
        let single_file_path = dir.path().join("single_file.txt");
        fs::create_dir(&test_dir1).unwrap();
        fs::create_dir(&test_dir2).unwrap();
        let file1_path = test_dir1.join("file1.txt");
        let file2_path = test_dir2.join("file2.txt");
        fs::File::create(&file1_path)
            .unwrap()
            .write_all(b"Contents of file1")
            .unwrap();
        fs::File::create(&file2_path)
            .unwrap()
            .write_all(b"Contents of file2")
            .unwrap();
        fs::File::create(&single_file_path)
            .unwrap()
            .write_all(b"Contents of single file")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir1).arg(&test_dir2).arg(&single_file_path);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test_dir1/file1.txt"))
            .stdout(predicate::str::contains("Contents of file1"))
            .stdout(predicate::str::contains("test_dir2/file2.txt"))
            .stdout(predicate::str::contains("Contents of file2"))
            .stdout(predicate::str::contains("single_file.txt"))
            .stdout(predicate::str::contains("Contents of single file"));
    }

    #[test]
    fn test_ignore_patterns() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        let file_to_ignore_path = test_dir.join("file_to_ignore.txt");
        let file_to_include_path = test_dir.join("file_to_include.txt");
        fs::File::create(&file_to_ignore_path)
            .unwrap()
            .write_all(b"This file should be ignored due to ignore patterns")
            .unwrap();
        fs::File::create(&file_to_include_path)
            .unwrap()
            .write_all(b"This file should be included")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg("--ignore").arg("*.txt");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("file_to_ignore.txt").not())
            .stdout(
                predicate::str::contains("This file should be ignored due to ignore patterns")
                    .not(),
            )
            .stdout(predicate::str::contains("file_to_include.txt").not());
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg("--ignore").arg("file_to_ignore.*");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("file_to_ignore.txt").not())
            .stdout(
                predicate::str::contains("This file should be ignored due to ignore patterns")
                    .not(),
            )
            .stdout(predicate::str::contains("test_dir/file_to_include.txt"))
            .stdout(predicate::str::contains("This file should be included"));
    }

    #[test]
    fn test_specific_extensions() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        let two_dir = test_dir.join("two");
        fs::create_dir_all(&two_dir).unwrap();
        let one_txt_path = test_dir.join("one.txt");
        let one_py_path = test_dir.join("one.py");
        let two_txt_path = two_dir.join("two.txt");
        let two_py_path = two_dir.join("two.py");
        let three_md_path = test_dir.join("three.md");
        fs::File::create(&one_txt_path)
            .unwrap()
            .write_all(b"This is one.txt")
            .unwrap();
        fs::File::create(&one_py_path)
            .unwrap()
            .write_all(b"This is one.py")
            .unwrap();
        fs::File::create(&two_txt_path)
            .unwrap()
            .write_all(b"This is two/two.txt")
            .unwrap();
        fs::File::create(&two_py_path)
            .unwrap()
            .write_all(b"This is two/two.py")
            .unwrap();
        fs::File::create(&three_md_path)
            .unwrap()
            .write_all(b"This is three.md")
            .unwrap();

        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg("-e").arg("py").arg("-e").arg("md");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(".txt").not())
            .stdout(predicate::str::contains("test_dir/one.py"))
            .stdout(predicate::str::contains("test_dir/two/two.py"))
            .stdout(predicate::str::contains("test_dir/three.md"));
    }

    #[test]
    fn test_mixed_paths_with_options() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        let gitignore_path = test_dir.join(".gitignore");
        let ignored_in_gitignore_path = test_dir.join("ignored_in_gitignore.txt");
        let hidden_ignored_in_gitignore_path = test_dir.join(".hidden_ignored_in_gitignore.txt");
        let included_path = test_dir.join("included.txt");
        let hidden_included_path = test_dir.join(".hidden_included.txt");
        let single_file_path = dir.path().join("single_file.txt");
        fs::create_dir(&test_dir).unwrap();
        fs::File::create(&gitignore_path)
            .unwrap()
            .write_all(b"ignored_in_gitignore.txt\n.hidden_ignored_in_gitignore.txt")
            .unwrap();
        fs::File::create(&ignored_in_gitignore_path)
            .unwrap()
            .write_all(b"This file should be ignored by .gitignore")
            .unwrap();
        fs::File::create(&hidden_ignored_in_gitignore_path)
            .unwrap()
            .write_all(b"This hidden file should be ignored by .gitignore")
            .unwrap();
        fs::File::create(&included_path)
            .unwrap()
            .write_all(b"This file should be included")
            .unwrap();
        fs::File::create(&hidden_included_path)
            .unwrap()
            .write_all(b"This hidden file should be included")
            .unwrap();
        fs::File::create(&single_file_path)
            .unwrap()
            .write_all(b"Contents of single file")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg(&single_file_path);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("ignored_in_gitignore.txt").not())
            .stdout(predicate::str::contains(".hidden_ignored_in_gitignore.txt").not())
            .stdout(predicate::str::contains("test_dir/included.txt"))
            .stdout(predicate::str::contains(".hidden_included.txt").not())
            .stdout(predicate::str::contains("single_file.txt"))
            .stdout(predicate::str::contains("Contents of single file"));
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir)
            .arg(&single_file_path)
            .arg("--include-hidden");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("ignored_in_gitignore.txt").not())
            .stdout(predicate::str::contains(".hidden_ignored_in_gitignore.txt").not())
            .stdout(predicate::str::contains("test_dir/included.txt"))
            .stdout(predicate::str::contains("test_dir/.hidden_included.txt"))
            .stdout(predicate::str::contains("single_file.txt"))
            .stdout(predicate::str::contains("Contents of single file"));
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir)
            .arg(&single_file_path)
            .arg("--ignore-gitignore");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(
                "test_dir/ignored_in_gitignore.txt",
            ))
            .stdout(predicate::str::contains(".hidden_ignored_in_gitignore.txt").not())
            .stdout(predicate::str::contains("test_dir/included.txt"))
            .stdout(predicate::str::contains(".hidden_included.txt").not())
            .stdout(predicate::str::contains("single_file.txt"))
            .stdout(predicate::str::contains("Contents of single file"));
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir)
            .arg(&single_file_path)
            .arg("--ignore-gitignore")
            .arg("--include-hidden");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(
                "test_dir/ignored_in_gitignore.txt",
            ))
            .stdout(predicate::str::contains(
                "test_dir/.hidden_ignored_in_gitignore.txt",
            ))
            .stdout(predicate::str::contains("test_dir/included.txt"))
            .stdout(predicate::str::contains("test_dir/.hidden_included.txt"))
            .stdout(predicate::str::contains("single_file.txt"))
            .stdout(predicate::str::contains("Contents of single file"));
    }

    #[test]
    fn test_binary_file_warning() {
        use predicates::str::is_match;
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        let binary_file_path = test_dir.join("binary_file.bin");
        let text_file_path = test_dir.join("text_file.txt");
        fs::File::create(&binary_file_path)
            .unwrap()
            .write_all(&[0xff])
            .unwrap();
        fs::File::create(&text_file_path)
            .unwrap()
            .write_all(b"This is a text file")
            .unwrap();
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test_dir/text_file.txt"))
            .stdout(predicate::str::contains("This is a text file"))
            .stdout(predicate::str::contains("\nbinary_file.bin").not())
            .stderr(
                is_match(
                    r"Warning: Skipping file .*test_dir/binary_file\.bin due to UnicodeDecodeError",
                )
                .unwrap(),
            );
    }

    #[test]
    fn test_output_option() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        let file1_path = test_dir.join("file1.txt");
        let file2_path = test_dir.join("file2.txt");
        fs::File::create(&file1_path)
            .unwrap()
            .write_all(b"Contents of file1.txt")
            .unwrap();
        fs::File::create(&file2_path)
            .unwrap()
            .write_all(b"Contents of file2.txt")
            .unwrap();
        let output_file_path = dir.path().join("output.txt");
        let mut cmd = Command::cargo_bin("proompt").unwrap();
        cmd.arg(&test_dir).arg("-o").arg(&output_file_path);
        cmd.assert().success().stdout("");
        let output_content = fs::read_to_string(&output_file_path).unwrap();
        assert!(output_content.contains("test_dir/file1.txt"));
        assert!(output_content.contains("Contents of file1.txt"));
        assert!(output_content.contains("test_dir/file2.txt"));
        assert!(output_content.contains("Contents of file2.txt"));
    }
}
