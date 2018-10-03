`dstate` quickly allows you to get user and kernel stacks for hung threads.
"Quickly" here is important, as getting stacks is by its very nature racy.

This is useful in order to debug the cause of becoming uninterruptible, and
finding code paths that are particularly susceptible to such states. An example
of where it can come in useful is documented in [this
article](https://chrisdown.name/2018/04/17/kernel-adventures-the-curious-case-of-squashfs-stalls.html).

# Requirements

Other than Rust, there are no requirements. An optional dependency is
[quickstack](https://github.com/yoshinorim/quickstack) if one wants userspace
stacks, but dstate will work without it, only outputting kernel threads.

# Example output

    $ dstate
    # 1459 (comm: utxvtd) (cmd: /usr/bin/urxvtd):

    Kernel stack:

    [<0>] __flush_work+0x10e/0x1c0
    [<0>] n_tty_read+0x2cd/0x8a0
    [<0>] tty_read+0x95/0x120
    [<0>] __vfs_read+0x36/0x180
    [<0>] vfs_read+0x8a/0x130
    [<0>] ksys_read+0x4f/0xb0
    [<0>] do_syscall_64+0x5b/0x170
    [<0>] entry_SYSCALL_64_after_hwframe+0x44/0xa9
    [<0>] 0xffffffffffffffff

    Userspace stack:

    Thread 1 (LWP 1459):
    #01  0x000055fbe1b9dbd6 in rxvt_term::pty_fill () from /usr/bin/urxvtd
    #02  0x000055fbe1b9fc58 in rxvt_term::pty_cb () from /usr/bin/urxvtd
    #03  0x000055fbe1bb86ac in ev_invoke_pending () from /usr/bin/urxvtd
    #04  0x000055fbe1bb8f51 in ev_run () from /usr/bin/urxvtd
    #05  0x000055fbe1b986c6 in main () from /usr/bin/urxvtd

    ---

    # 296 (comm: btrfs-transacti) (cmd: ):

    Kernel stack:

    [<0>] io_schedule+0x12/0x40
    [<0>] write_all_supers+0x418/0xa70 [btrfs]
    [<0>] btrfs_commit_transaction+0x52c/0x8a0 [btrfs]
    [<0>] transaction_kthread+0x13f/0x170 [btrfs]
    [<0>] kthread+0x112/0x130
    [<0>] ret_from_fork+0x35/0x40
    [<0>] 0xffffffffffffffff

    ---

    # 21277 (comm: vim) (cmd: vim)

    Kernel stack:

    [<0>] io_schedule+0x12/0x40
    [<0>] write_all_supers+0x418/0xa70 [btrfs]
    [<0>] btrfs_sync_log+0x5e8/0x970 [btrfs]
    [<0>] btrfs_sync_file+0x22a/0x430 [btrfs]
    [<0>] do_fsync+0x38/0x70
    [<0>] __x64_sys_fsync+0x10/0x20
    [<0>] do_syscall_64+0x5b/0x170
    [<0>] entry_SYSCALL_64_after_hwframe+0x44/0xa9
    [<0>] 0xffffffffffffffff

    Userspace stack:

    Thread 1 (LWP 21277):
    #01  0x00005645eed9486c in __libc_csu_fini () from /usr/bin/vim
