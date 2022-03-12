pub mod poll {
    use libc::poll;
    use std::net::TcpListener;
    use std::os::unix::io::{AsRawFd, RawFd};

    pub type EventFlags = libc::c_short;
    pub const READ: EventFlags = libc::POLLIN | libc::POLLPRI;
    pub const WRITE: EventFlags = libc::POLLOUT | libc::POLLWRBAND;
    pub const HANGUP: EventFlags = libc::POLLRDHUP;
    pub const ERROR: EventFlags = libc::POLLERR;
    pub const INVALID: EventFlags = libc::POLLNVAL;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, Default)]
    pub struct PollFd {
        fd: RawFd,
        events: EventFlags,
        revents: EventFlags,
    }

    pub struct IoEvent<K> {
        pub key: K,
        events: EventFlags,
    }

    impl<K> IoEvent<K> {
        pub fn is_any(&self, flags: EventFlags) -> bool {
            flags & self.events != 0
        }

        pub fn is_read(&self) -> bool {
            (self.events & READ) != 0
        }

        pub fn is_write(&self) -> bool {
            (self.events & WRITE) != 0
        }
    }

    pub struct Registry<K> {
        key_index: Vec<K>,
        poll_fds: Vec<PollFd>,
    }

    impl<K: Eq + Clone> Registry<K> {
        pub fn register(&mut self, key: K, f: &impl AsRawFd, event_type: EventFlags) {
            self.key_index.push(key);
            let poll_fd = PollFd {
                fd: f.as_raw_fd(),
                events: event_type,
                revents: 0,
            };
            self.poll_fds.push(poll_fd)
        }

        pub fn unregister(&mut self, key: &K) {
            if let Some(ix) = self.key_index.iter().position(|k| k == key) {
                self.key_index.swap_remove(ix);
                self.poll_fds.swap_remove(ix);
            }
        }

        pub fn new() -> Self {
            Self {
                key_index: Vec::with_capacity(8),
                poll_fds: Vec::with_capacity(8),
            }
        }

        pub fn wait(&mut self, filled_events: &mut Vec<IoEvent<K>>) {
            for poll_fd in &mut self.poll_fds {
                poll_fd.revents = 0;
            }
            let r = self.poll(-1);
            filled_events.clear();
            for (i, poll_fd) in self.poll_fds.iter().enumerate() {
                if poll_fd.revents != 0 {
                    filled_events.push(IoEvent {
                        key: self.key_index[i].clone(),
                        events: poll_fd.revents,
                    });
                }
            }
        }

        fn poll(&mut self, timeout: i32) -> i32 {
            unsafe {
                libc::poll(
                    self.poll_fds.as_mut_ptr() as *mut libc::pollfd,
                    self.poll_fds.len() as libc::nfds_t,
                    timeout,
                )
            }
        }
    }
}
