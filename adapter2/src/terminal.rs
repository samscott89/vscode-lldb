use crate::error::Error;
use log::debug;
use std::io::{self, BufRead};
use std::net;

#[cfg(windows)]
extern "stdcall" {
    fn FreeConsole() -> u32;
    fn AttachConsole(pid: u32) -> u32;
    fn GetLastError() -> u32;
}

pub struct Terminal {
    connection: net::TcpStream,
    data: String,
}

impl Terminal {
    pub fn create<F>(run_in_terminal: F) -> Result<Self, Error>
    where
        F: FnOnce(Vec<String>),
    {
        let mut listener = net::TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        // Opens TCP connection, send output of `tty`, wait till the socket gets closed from our end
        let mut executable = std::env::current_exe()?;

        let args =
            vec![executable.to_str().unwrap().into(), "terminal-agent".into(), format!("--port={}", addr.port())];

        run_in_terminal(args);

        let (stream, _) = listener.accept()?;
        let stream2 = stream.try_clone()?;

        let mut reader = io::BufReader::new(stream);
        let mut data = String::new();
        reader.read_line(&mut data)?;

        Ok(Terminal {
            connection: stream2,
            data: data.trim().to_owned(),
        })
    }

    pub fn input_devname(&self) -> &str {
        if cfg!(windows) {
            "CONIN$"
        } else {
            &self.data
        }
    }

    pub fn output_devname(&self) -> &str {
        if cfg!(windows) {
            "CONOUT$"
        } else {
            &self.data
        }
    }

    pub fn attach<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Windows does not have an API for launching a child process attached to another console.
        // Instead,
        #[cfg(windows)]
        {
            let pid = self.data.parse::<u32>().unwrap();
            unsafe {
                dbg!(FreeConsole());
                dbg!(AttachConsole(pid));
            }
            let result = f();
            unsafe {
                dbg!(FreeConsole());
            }
            result
        }

        #[cfg(not(windows))]
        f()
    }
}
