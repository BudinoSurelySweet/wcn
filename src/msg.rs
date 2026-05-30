#[macro_export]
macro_rules! success {
    ($msg:expr) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Success".bright_green().bold(), $msg);
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Success".bright_green().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! err {
    ($msg:expr) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Error".bright_red().bold(), $msg);
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Error".bright_red().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! pending {
    ($msg:expr) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Pending".bright_green().bold(), $msg);
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Pending".bright_green().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! running {
    ($msg:expr) => {{
        use colored::Colorize;

        println!("{:>12} `{}`", "Running".bright_green().bold(), $msg);
    }};
}

#[macro_export]
macro_rules! arguments {
    ($msg:expr) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Arguments".bright_green().bold(), $msg);
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::Colorize;

            println!("{:>12} {}", "Arguments".bright_green().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! profile {
    ($msg:expr) => {{
        use colored::Colorize;

        println!("{:>12} {}", "Profile".bright_green().bold(), $msg);
    }};
}
