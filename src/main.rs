use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, getgid, getppid, getuid, ForkResult, Gid, Uid};
use std::ffi::CString;
use std::fs;
use std::process::{exit, Command};

fn clone_user_namespace(to_uid: Uid, to_gid: Gid, clone_mount: bool) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let mut flags = CloneFlags::CLONE_NEWUSER;
            if clone_mount {
                flags.insert(CloneFlags::CLONE_NEWNS);
            }
            unshare(flags).unwrap();
            waitpid(child, None).unwrap();

            if clone_mount {
                // to prevent propagation
                let flags = MsFlags::MS_REC | MsFlags::MS_SLAVE;
                mount(None::<&str>, "/", None::<&str>, flags, None::<&str>).unwrap();
            }
        }
        Ok(ForkResult::Child) => {
            let ppid = getppid();
            let from_uid = getuid();
            let from_gid = getgid();
            // TODO: wait to unshare on parrent
            Command::new("newuidmap")
                .args([
                    ppid.to_string(),
                    to_uid.to_string(),
                    from_uid.to_string(),
                    "1".to_string(),
                ])
                .output()
                .unwrap();
            Command::new("newgidmap")
                .args([
                    ppid.to_string(),
                    to_gid.to_string(),
                    from_gid.to_string(),
                    "1".to_string(),
                ])
                .output()
                .unwrap();
            exit(0);
        }
        Err(_) => panic!("Fork failed"),
    }
}

fn main() {
    let root_dir = dirs::home_dir().unwrap();
    let holo_dir = root_dir.join(".holo");
    let layers_dir = holo_dir.join("layers");

    let uid = getuid();
    let gid = getgid();
    clone_user_namespace(0.into(), 0.into(), true);

    let layer_dir = layers_dir.join("hoge");
    let entries_dir = layer_dir.join("entries");
    for dir in fs::read_dir(entries_dir).unwrap() {
        let dir = dir.unwrap();
        let name = dir.file_name();

        let flags = MsFlags::MS_BIND;
        mount(
            Some(&dir.path().join("filesystem")),
            &root_dir.join(name),
            None::<&str>,
            flags,
            None::<&str>,
        )
        .unwrap();
    }

    clone_user_namespace(uid, gid, false);

    let shell = CString::new("fish").unwrap();
    execvp(&shell, &[shell.clone()]).unwrap();
}
