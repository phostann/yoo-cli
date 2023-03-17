use anyhow::Result;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub(crate) fn loading(tip: &str) -> Result<ProgressBar> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]",
                "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
            ]),
    );

    pb.set_message(tip.to_string());

    Ok(pb)
}

// test
#[cfg(test)]
mod test {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Duration;

    #[test]
    fn test_indicatif() {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                // For more spinners check out the cli-spinners project:
                // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                .tick_strings(&[
                    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]",
                    "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
                ]),
        );

        pb.set_message("Loading...".to_string());
        // sleep 3 seconds
        std::thread::sleep(Duration::from_secs(3));
        pb.finish_and_clear()
    }
}
