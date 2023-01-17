use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{execvp, fork, getppid, ForkResult};
use std::process::Command;

use std::ffi::CString;
fn main() {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS;
            unshare(flags).unwrap();

            let flags = MsFlags::MS_BIND;
            mount(
                Some("/home/satoshi/tmp/hoge"),
                "/home/satoshi/tmp/fuga",
                None::<&str>,
                flags,
                None::<&str>,
            )
            .unwrap();

            // TODO: wait to create id mapping on child
            let shell = CString::new("fish").unwrap();
            execvp(&shell, &[shell.clone()]).unwrap();
        }
        Ok(ForkResult::Child) => {
            let pid = getppid();
            // TODO: wait to unshare on parrent
            Command::new("newuidmap")
                .args([
                    pid.to_string(),
                    "0".to_string(),
                    "1000".to_string(),
                    "1".to_string()
                ])
                .output()
                .unwrap();
            Command::new("newgidmap")
                .args([
                    pid.to_string(),
                    "0".to_string(),
                    "1000".to_string(),
                    "1".to_string(),
                ])
                .output()
                .unwrap();
        }
        Err(_) => println!("Fork failed"),
    }
}
