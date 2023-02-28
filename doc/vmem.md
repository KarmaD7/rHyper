路径：
rvm/arch/aarch64/ept.rs 定义 stage 2 页表
hypervisor/mm/gpm.rs 映射地址段
hypervisor/mm/gconfig.rs 一些地址配置

其它：
rvm/arch/aarch64/vcpu.rs 保存 vcpu 页表相关状态

stage 2 页表的 attribute: D5 2603 & 2604 页

相关寄存器：VTTBR_EL2, VTCR_EL2![41677153038_.pic](/Users/dsf/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9/96e25f9ec48c23bec2e2af4f95812295/Message/MessageTemp/a2d5ed7aac15fe3d5db1da0afda5cf38/Image/41677153038_.pic.jpg)

![51677154390_.pic](/Users/dsf/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9/96e25f9ec48c23bec2e2af4f95812295/Message/MessageTemp/a2d5ed7aac15fe3d5db1da0afda5cf38/Image/51677154390_.pic.jpg)

![61677154405_.pic](/Users/dsf/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9/96e25f9ec48c23bec2e2af4f95812295/Message/MessageTemp/a2d5ed7aac15fe3d5db1da0afda5cf38/Image/61677154405_.pic.jpg)

![截屏2023-02-25 15.43.58](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-02-25 15.43.58.png)

![截屏2023-02-25 15.57.58](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-02-25 16.18.41.png)

![截屏2023-02-25 16.40.26](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-02-25 16.40.26.png)

![image-20230226232401106](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-02-27 22.00.49.png)

 HPFAR_EL2, Hypervisor IPA Fault Address Register

![截屏2023-02-27 22.06.30](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-02-27 22.08.38.png)
