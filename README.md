`dstate` quickly allows you to get user and kernel stacks for hung threads.
"Quickly" here is important, as getting stacks is by its very nature racy.

# Example output:

    # 1459 (/usr/bin/urxvtd):

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
