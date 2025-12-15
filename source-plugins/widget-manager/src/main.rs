use std::{
    env, io,
    process::{Child, Command},
};

enum Action {
    Open,
    Close,
    Toggle,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <open|close> <window_name>", args[0]);
        std::process::exit(1);
    }

    let action = match args[1].as_str() {
        "open" => Action::Open,
        "close" => Action::Close,
        "toggle" => Action::Toggle,
        _ => {
            eprintln!("Error: Invalid action '{}'", args[1]);
            std::process::exit(1);
        }
    };

    let target = &args[2];
    let reveal_var = args.get(3).map(|s| s.as_str());

    execute(action, target, reveal_var)
}

fn execute(action: Action, target: &str, reveal_var: Option<&str>) -> io::Result<()> {
    match action {
        Action::Open => open_window(target, reveal_var),
        Action::Close => close_window(target, reveal_var),
        Action::Toggle => {
            if is_window_active(target)? {
                close_window(target, reveal_var)
            } else {
                open_window(target, reveal_var)
            }
        }
    }
}

fn open_window(target: &str, reveal_var: Option<&str>) -> io::Result<()> {
    if is_window_active(target)? {
        println!("Window '{}' is already active.", target);
        return Ok(());
    }

    run_eww(&["open", target])?;
    println!("Opened window '{}'.", target);

    if let Some(var) = reveal_var {
        run_eww(&["update", &format!("{}=true", var)])?;
        println!("Updated {} to true.", var);
    }

    Ok(())
}

fn close_window(target: &str, reveal_var: Option<&str>) -> io::Result<()> {
    if !is_window_active(target)? {
        println!("Window '{}' is not active.", target);
        return Ok(());
    }

    if let Some(var) = reveal_var {
        run_eww(&["update", &format!("{}=false", var)])?;
        Command::new("sh")
            .arg("-c")
            .arg(format!("sleep 0.2 && eww close {}", target))
            .spawn()?;
        println!("Closed '{}' with animation.", target);
    } else {
        run_eww(&["close", target])?;
        println!("Closed '{}' immediately.", target);
    }

    Ok(())
}

fn is_window_active(target: &str) -> io::Result<bool> {
    let output = Command::new("eww").arg("active-windows").output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);

                let exists = stdout.lines().any(|line| {
                    let name = line.split(':').next().unwrap_or(line).trim();
                    name == target
                });

                Ok(exists)
            } else {
                Ok(false)
            }
        }
        Err(_) => {
            eprintln!("Error: Could not execute 'eww'. Is it installed?");
            Ok(false)
        }
    }
}

fn run_eww(args: &[&str]) -> io::Result<Child> {
    let mut child = Command::new("eww").args(args).spawn()?;
    child.wait()?;
    Ok(child)
}
