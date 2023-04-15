#[cfg(test)]
mod tests {
    use std::fs;
    use whisperfish::store::Storage;

    #[test]
    #[rustfmt::skip]
    fn log_rotation() {
        let temp_path = tempfile::tempdir().unwrap();
        let temp_pathbuf = temp_path.path().to_owned();

        fs::write(temp_pathbuf.join("dont-touch-me.tmp"), "ever").unwrap();

        // Same format as in harbour-whisperfish main.rs
        fs::write(temp_pathbuf.join("harbour-whisperfish.20001231_124500.log"), "log contents").unwrap();
        fs::write(temp_pathbuf.join("harbour-whisperfish.20001231_134500.log"), "log contents").unwrap();
        fs::write(temp_pathbuf.join("harbour-whisperfish.20001231_144500.log"), "log contents").unwrap();
        fs::write(temp_pathbuf.join("harbour-whisperfish.20001231_154500.log"), "log contents").unwrap();
        fs::write(temp_pathbuf.join("harbour-whisperfish.20001231_164500.log"), "log contents").unwrap();

        fs::write(temp_pathbuf.join("leave-me-alone.tmp"), "too").unwrap();

        // Same as in harbour-whisperfish main.rs
        const LOGFILE_REGEX: &str = r"harbour-whisperfish.\d{8}_\d{6}\.log";

        assert!(!Storage::clear_old_logs(&temp_pathbuf, 1, LOGFILE_REGEX));

        let nonexistent = temp_pathbuf.join("foobar");
        assert!(!Storage::clear_old_logs(&nonexistent, 5, LOGFILE_REGEX));

        assert!(Storage::clear_old_logs(&temp_pathbuf, 5, LOGFILE_REGEX));
        assert_eq!(fs::read_dir(&temp_pathbuf).unwrap().count(), 7);

        assert!(Storage::clear_old_logs(&temp_pathbuf, 3, LOGFILE_REGEX));
        assert_eq!(fs::read_dir(&temp_pathbuf).unwrap().count(), 5);

        let mut remaining = fs::read_dir(&temp_pathbuf).unwrap();
        assert_eq!("leave-me-alone.tmp",                      remaining.next().unwrap().unwrap().file_name().to_str().unwrap());
        assert_eq!("harbour-whisperfish.20001231_164500.log", remaining.next().unwrap().unwrap().file_name().to_str().unwrap());
        assert_eq!("harbour-whisperfish.20001231_154500.log", remaining.next().unwrap().unwrap().file_name().to_str().unwrap());
        assert_eq!("harbour-whisperfish.20001231_144500.log", remaining.next().unwrap().unwrap().file_name().to_str().unwrap());
        // deleted: harbour-whisperfish.20001231_134500.log
        // deleted: harbour-whisperfish.20001231_124500.log
        assert_eq!("dont-touch-me.tmp",                       remaining.next().unwrap().unwrap().file_name().to_str().unwrap());
        assert!(remaining.next().is_none());

        drop(temp_path);
    }
}
