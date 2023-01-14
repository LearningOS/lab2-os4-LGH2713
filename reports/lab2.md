# Lab2
### 基础概念

+ 页面（Page）：每个应用的地址空间被分成小块 。
+ 页帧（Frame）：可用物理内存被分成的小块。
+ 虚拟页号（VPN，Virtual Page Number）
+ 物理页号（PPN, Physical Page Number）
+ 页表（Page Table）：每个应用都有一个页表，用于VPN与PPN转换，其中每个VPN也有一组保护位（rwx）表示权限，页表 同样存放在内存中。
+ 逻辑段：由多个虚拟页面组成，在应用视角是一块连续的内存。