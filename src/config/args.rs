/// Handle Command Line Arguments
pub struct Args;

impl Args {
    pub fn check_args(config: &mut super::SignalConfig) {
        let mut ignored = 0;

        for arg in std::env::args() {
            if arg == "autostart" {
                config.autostart = true;
            } else if arg == "-v" || arg == "--verbose" {
                config.verbose = true;
            } else {
                ignored += 1;
            }
        }

        if ignored > 1 {
            eprintln!("{} console arguments ignored by Whisperfish.", ignored - 1);
            eprintln!("They might be handled by Qt.");
        }
    }
}
