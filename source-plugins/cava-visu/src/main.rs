use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("cava -p ~/.config/cava/config_eww")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start cava");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut output_handle = io::stdout();

    let bars = [" ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let max_val = 255.0;
    let max_idx = (bars.len() - 1) as f32;

    for l in reader.lines().map_while(Result::ok) {
        let values: Vec<&str> = l.trim().split(';').filter(|p| !p.is_empty()).collect();

        for (i, part) in values.iter().enumerate() {
            if let Ok(val) = part.parse::<f32>() {
                let mut idx = ((val / max_val) * max_idx) as usize;

                if idx >= bars.len() {
                    idx = bars.len() - 1;
                }

                print!("{}", bars[idx]);

                if i < values.len() - 1 {
                    print!(" ");
                }
            }
        }

        println!();
        output_handle.flush().unwrap_or(());
    }

    let _ = child.wait();
}
