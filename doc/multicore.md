- lock

- 启动：psci smc call
- callee saved registers: X19 - X28

- percpu

启动时只启动一个 guest 的主核 -> guest 的主核启动 guest 的从核 -> 用 guest 的主核传进来的 guest entry, 从 global 拿 npt root， run
不难改
困难的还是 virtio = =